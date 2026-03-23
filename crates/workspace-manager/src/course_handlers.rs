use crate::helpers::{check_scope, require_auth, verify_workspace_ownership};
use crate::{WorkspaceManagerState, file_editor};
use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, Deserialize)]
pub(crate) struct SyncCourseYamlRequest {
    pub folder_path: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SyncCourseYamlResponse {
    /// Relative path of the course.yaml file within the workspace.
    pub file_path: String,
    /// true = file was created fresh; false = existing file was updated.
    pub created: bool,
    /// Number of new lessons appended (0 on fresh create means "all").
    pub added: usize,
}

/// POST /api/workspaces/{workspace_id}/course/sync-yaml
///
/// Creates or updates `course.yaml` for a course folder.
/// - If the file does not exist, generates a full yaml from all discovered .md files.
/// - If the file exists, appends only lessons not already listed, preserving everything else.
pub(crate) async fn sync_course_yaml(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<SyncCourseYamlRequest>,
) -> Result<Json<SyncCourseYamlResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_abs = file_editor::safe_resolve_pub(&workspace_root, &req.folder_path)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !folder_abs.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Collect all .md/.mdx files, grouped by top-level subfolder
    let mut module_files: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for entry in walkdir::WalkDir::new(&folder_abs)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" && ext != "mdx" {
            continue;
        }
        let rel = match path.strip_prefix(&folder_abs) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let parts: Vec<&str> = rel.splitn(2, '/').collect();
        let module_key = if parts.len() > 1 { parts[0].to_string() } else { String::new() };
        module_files.entry(module_key).or_default().push(rel);
    }

    // Sort lessons within each module alphabetically (natural order)
    for files in module_files.values_mut() {
        files.sort();
    }

    let yaml_rel = format!("{}/course.yaml", req.folder_path.trim_end_matches('/'));
    let yaml_abs = workspace_root.join(&yaml_rel);

    if yaml_abs.exists() {
        // ── Update mode: append only new lessons ────────────────────────────
        let existing = std::fs::read_to_string(&yaml_abs)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Collect all .md/.mdx paths already referenced in the yaml.
        // Simple text scan — no full parse, so comments and custom formatting are preserved.
        // Matches two patterns:
        //   lessons map key:   `  some/path.md:`   (2-space indent, ends with colon)
        //   course-processor:  `file: some/path.md`
        let referenced: std::collections::HashSet<String> = {
            let mut set = std::collections::HashSet::new();
            for line in existing.lines() {
                let trimmed = line.trim();
                // Map key form: "path/to/file.md:" (no leading dash)
                if !trimmed.starts_with('-') && !trimmed.starts_with('#') {
                    if let Some(key) = trimmed.strip_suffix(':') {
                        let key = key.trim().trim_matches('"').trim_matches('\'');
                        if key.ends_with(".md") || key.ends_with(".mdx") {
                            set.insert(key.to_string());
                        }
                    }
                }
                // file: field form
                if let Some(rest) = trimmed.strip_prefix("file:") {
                    let val = rest.trim().trim_matches('"').trim_matches('\'');
                    if val.ends_with(".md") || val.ends_with(".mdx") {
                        set.insert(val.to_string());
                    }
                }
            }
            set
        };

        let mut appended: Vec<String> = Vec::new();
        for files in module_files.values() {
            for rel in files {
                if !referenced.contains(rel.as_str()) {
                    appended.push(rel.clone());
                }
            }
        }

        if appended.is_empty() {
            return Ok(Json(SyncCourseYamlResponse {
                file_path: yaml_rel,
                created: false,
                added: 0,
            }));
        }

        // Append new lessons as a commented block at the end of the file
        let mut out = existing.trim_end().to_string();
        out.push_str("\n\n  # — new lessons (added by sync) —\n");
        for rel in &appended {
            let stem = std::path::Path::new(rel)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(rel);
            let title = stem
                .replace(['-', '_'], " ")
                .split_whitespace()
                .enumerate()
                .map(|(i, w)| {
                    if i == 0 {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    } else {
                        w.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            out.push_str(&format!("  {}:\n    title: \"{}\"\n    order: 999\n", rel, title));
        }
        out.push('\n');

        std::fs::write(&yaml_abs, &out).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(SyncCourseYamlResponse {
            file_path: yaml_rel,
            created: false,
            added: appended.len(),
        }))
    } else {
        // ── Create mode: generate full yaml ─────────────────────────────────
        let folder_name = folder_abs
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Course");
        let title = folder_name
            .replace(['-', '_'], " ")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut yaml = format!(
            "title: \"{title}\"\ndescription: \"\"\ninstructor: \"\"\nlevel: \"beginner\"\n\n"
        );

        // Modules block
        yaml.push_str("modules:\n");
        let mut module_order = 1i32;
        for (module_key, _) in &module_files {
            let mod_title = if module_key.is_empty() {
                "Introduction".to_string()
            } else {
                module_key
                    .replace(['-', '_'], " ")
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            yaml.push_str(&format!(
                "  - path: \"{}\"\n    title: \"{}\"\n    order: {}\n\n",
                module_key, mod_title, module_order
            ));
            module_order += 1;
        }

        // Lessons block
        yaml.push_str("lessons:\n");
        for (module_key, files) in &module_files {
            if !module_key.is_empty() {
                yaml.push_str(&format!("  # {}\n", module_key));
            } else {
                yaml.push_str("  # root-level lessons\n");
            }
            for (lesson_order, rel) in files.iter().enumerate() {
                let stem = std::path::Path::new(rel)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(rel);
                let lesson_title = stem
                    .replace(['-', '_'], " ")
                    .split_whitespace()
                    .enumerate()
                    .map(|(i, w)| {
                        if i == 0 {
                            let mut c = w.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().to_string() + c.as_str(),
                            }
                        } else {
                            w.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                yaml.push_str(&format!(
                    "  {}:\n    title: \"{}\"\n    order: {}\n",
                    rel,
                    lesson_title,
                    lesson_order + 1
                ));
            }
            yaml.push('\n');
        }

        std::fs::write(&yaml_abs, &yaml).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(SyncCourseYamlResponse {
            file_path: yaml_rel,
            created: true,
            added: module_files.values().map(|v| v.len()).sum(),
        }))
    }
}

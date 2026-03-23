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
pub(crate) struct SyncPresentationYamlRequest {
    pub folder_path: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SyncPresentationYamlResponse {
    pub file_path: String,
    pub created: bool,
}

/// POST /api/workspaces/{workspace_id}/presentation/sync-yaml
///
/// Creates `presentation.yaml` and an initial `slides.md` if they don't exist.
pub(crate) async fn sync_presentation_yaml(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<SyncPresentationYamlRequest>,
) -> Result<Json<SyncPresentationYamlResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let folder_abs = file_editor::safe_resolve_pub(&workspace_root, &req.folder_path)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !folder_abs.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    let folder_name = folder_abs
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Presentation")
        .to_string();

    let folder_prefix = req.folder_path.trim_end_matches('/');

    // Create presentation.yaml if absent
    let yaml_rel = format!("{}/presentation.yaml", folder_prefix);
    let yaml_abs = workspace_root.join(&yaml_rel);
    let mut created = false;
    if !yaml_abs.exists() {
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
        let content = format!(
            "title: \"{}\"\ntheme: white\ntransition: slide\nshow_progress: true\nshow_slide_number: all\nloop: false\nauto_slide: 0\n",
            title
        );
        std::fs::write(&yaml_abs, &content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        created = true;
    }

    // Create slides.md if absent
    let slides_rel = format!("{}/slides.md", folder_prefix);
    let slides_abs = workspace_root.join(&slides_rel);
    if !slides_abs.exists() {
        let title = folder_name.replace(['-', '_'], " ");
        let content = format!(
            "# {}\n\nYour first slide\n\n---\n\n## Second Slide\n\nAdd more slides separated by `---`\n",
            title
        );
        std::fs::write(&slides_abs, &content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return the yaml file path so the UI can open it in the editor
    Ok(Json(SyncPresentationYamlResponse {
        file_path: yaml_rel,
        created,
    }))
}

// ── Generate slides from course ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct GeneratePresentationFromCourseRequest {
    pub course_folder: String,
    pub target_folder: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GeneratePresentationFromCourseResponse {
    pub file_path: String,
    pub slide_count: usize,
    pub created: bool,
}

/// POST /api/workspaces/{workspace_id}/presentation/generate-from-course
///
/// Reads an existing course folder and generates a `slides.md` from its content.
pub(crate) async fn generate_presentation_from_course(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(req): Json<GeneratePresentationFromCourseRequest>,
) -> Result<Json<GeneratePresentationFromCourseResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let course_abs = file_editor::safe_resolve_pub(&workspace_root, &req.course_folder)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if !course_abs.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    let target_prefix = req
        .target_folder
        .as_deref()
        .unwrap_or(&req.course_folder)
        .trim_end_matches('/')
        .to_string();

    let target_abs = file_editor::safe_resolve_pub(&workspace_root, &target_prefix)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !target_abs.exists() {
        std::fs::create_dir_all(&target_abs).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Collect .md files grouped by top-level subfolder (same logic as sync_course_yaml)
    let mut module_files: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for entry in walkdir::WalkDir::new(&course_abs)
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
        // Skip slides.md itself to avoid self-reference
        if path.file_name().and_then(|n| n.to_str()) == Some("slides.md") {
            continue;
        }
        let rel = match path.strip_prefix(&course_abs) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let parts: Vec<&str> = rel.splitn(2, '/').collect();
        let module_key = if parts.len() > 1 { parts[0].to_string() } else { String::new() };
        module_files.entry(module_key).or_default().push(rel);
    }

    for files in module_files.values_mut() {
        files.sort();
    }

    let mut slides: Vec<String> = Vec::new();

    for (module_key, files) in &module_files {
        if !module_key.is_empty() {
            let title = module_key
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
            slides.push(format!("# {}", title));
        }
        for rel in files {
            let content = std::fs::read_to_string(course_abs.join(rel)).unwrap_or_default();
            slides.push(content);
        }
    }

    let slides_md = slides.join("\n\n---\n\n");
    let slide_count = slides_md.split("\n---\n").count();

    let slides_rel = format!("{}/slides.md", target_prefix);
    let slides_abs = workspace_root.join(&slides_rel);
    let created = !slides_abs.exists();
    std::fs::write(&slides_abs, &slides_md).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GeneratePresentationFromCourseResponse {
        file_path: slides_rel,
        slide_count,
        created,
    }))
}

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String, // relative to workspace root
    pub size: u64,
    pub size_str: String,
    pub mime_type: String,
    pub icon: String,
    pub modified: String,
    pub is_editable: bool,
}

#[derive(Clone, Debug)]
pub struct FolderEntry {
    pub name: String,
    pub path: String, // relative to workspace root
    pub file_count: usize,
    /// Folder type id, e.g. "js-tool". None if untyped (default).
    pub folder_type: Option<String>,
    /// Hex color from the type definition, e.g. "#F59E0B".
    pub type_color: Option<String>,
    /// Icon name from the type definition, e.g. "code-2".
    pub type_icon: Option<String>,
    /// Display name of the type, e.g. "JavaScript Tool Collection".
    pub type_name: Option<String>,
    /// URL to serve the folder's thumbnail/icon image, if one exists.
    pub icon_url: Option<String>,
}

pub struct DirListing {
    pub folders: Vec<FolderEntry>,
    pub files: Vec<FileEntry>,
}

/// Resolve and validate a subpath within a workspace root.
/// Prevents path traversal attacks.
fn safe_resolve(workspace_root: &Path, subpath: &str) -> Result<PathBuf> {
    let clean = subpath
        .trim_start_matches('/')
        .trim_end_matches('/');

    // Reject obvious traversal
    for segment in clean.split('/') {
        if segment == ".." || segment == "." {
            bail!("Path traversal not allowed");
        }
    }

    let target = if clean.is_empty() {
        workspace_root.to_path_buf()
    } else {
        workspace_root.join(clean)
    };

    // Ensure resolved path is still within workspace root
    let canonical_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let canonical_target = target
        .canonicalize()
        .unwrap_or_else(|_| target.clone());

    if !canonical_target.starts_with(&canonical_root) {
        bail!("Path escapes workspace root");
    }

    Ok(target)
}

/// List directory contents (one level deep).
pub fn list_dir(workspace_root: &Path, subpath: &str) -> Result<DirListing> {
    let dir = safe_resolve(workspace_root, subpath)?;

    if !dir.exists() || !dir.is_dir() {
        bail!("Directory not found: {:?}", dir);
    }

    let mut folders = Vec::new();
    let mut files = Vec::new();

    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort: folders first, then files, alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/folders (starting with .)
        if name.starts_with('.') {
            continue;
        }

        let file_type = entry.file_type()?;
        let entry_rel_path = if subpath.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", subpath.trim_end_matches('/'), name)
        };

        if file_type.is_dir() {
            let file_count = WalkDir::new(entry.path())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .count();

            folders.push(FolderEntry {
                name,
                path: entry_rel_path,
                file_count,
                folder_type: None,
                type_color: None,
                type_icon: None,
                type_name: None,
                icon_url: None,
            });
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            let size = metadata.len();
            let mime_type = mime_guess::from_path(&name)
                .first_or_text_plain()
                .to_string();
            let is_editable = is_text_mime(&mime_type);
            let icon = file_icon(&mime_type).to_string();
            let size_str = format_size(size);
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| format_modified(d.as_secs()))
                })
                .unwrap_or_default();

            files.push(FileEntry {
                name,
                path: entry_rel_path,
                size,
                size_str,
                mime_type,
                icon,
                modified,
                is_editable,
            });
        }
    }

    Ok(DirListing { folders, files })
}

/// Get recently modified files across the entire workspace (up to `limit`).
pub fn recent_files(workspace_root: &Path, limit: usize) -> Vec<FileEntry> {
    if !workspace_root.exists() {
        return Vec::new();
    }

    let mut file_entries: Vec<(u64, FileEntry)> = WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            // Skip hidden files
            !e.file_name()
                .to_string_lossy()
                .starts_with('.')
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified_secs = metadata
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();

            let name = entry.file_name().to_string_lossy().to_string();
            let rel_path = entry
                .path()
                .strip_prefix(workspace_root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");

            let mime_type = mime_guess::from_path(&name)
                .first_or_text_plain()
                .to_string();
            let is_editable = is_text_mime(&mime_type);
            let icon = file_icon(&mime_type).to_string();
            let size = metadata.len();
            let size_str = format_size(size);

            Some((
                modified_secs,
                FileEntry {
                    name,
                    path: rel_path,
                    size,
                    size_str,
                    mime_type,
                    icon,
                    modified: format_modified(modified_secs),
                    is_editable,
                },
            ))
        })
        .collect();

    // Sort by modified time descending
    file_entries.sort_by(|a, b| b.0.cmp(&a.0));
    file_entries
        .into_iter()
        .take(limit)
        .map(|(_, f)| f)
        .collect()
}

/// Check whether a folder contains a `thumbnail*` or `icon*` image file.
pub fn folder_has_icon(folder_abs: &Path) -> bool {
    icon_file_path(folder_abs).is_some()
}

/// Return the path to the icon/thumbnail file inside a folder, if any.
/// Matches exact names (thumbnail.png, icon.svg) and prefixed names (thumbnail_preview.png, icon_app.png).
/// Exact names are preferred over prefixed variants.
pub fn icon_file_path(folder_abs: &Path) -> Option<std::path::PathBuf> {
    let exts: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg"];
    let entries = std::fs::read_dir(folder_abs).ok()?;
    let mut candidates: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            let ext = std::path::Path::new(&name)
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if !exts.contains(&ext) {
                return None;
            }
            let stem = std::path::Path::new(&name)
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("");
            if stem == "thumbnail" || stem.starts_with("thumbnail_")
                || stem == "icon" || stem.starts_with("icon_")
            {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();
    // Prefer exact names over prefixed variants
    candidates.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == "thumbnail" || stem == "icon" { 0u8 } else { 1u8 }
    });
    candidates.into_iter().next()
}


pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn file_icon(mime: &str) -> &'static str {
    if mime.starts_with("text/markdown") || mime.contains("markdown") {
        "file-text"
    } else if mime.contains("yaml") || mime.contains("json") {
        "settings-2"
    } else if mime.contains("bpmn") || mime.contains("xml") {
        "git-branch"
    } else if mime.starts_with("text/") {
        "file-code"
    } else if mime.starts_with("image/") {
        "image"
    } else if mime.contains("pdf") {
        "file-type-2"
    } else {
        "paperclip"
    }
}

fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/")
        || mime == "application/json"
        || mime == "application/yaml"
        || mime == "application/x-yaml"
        || mime.contains("xml")
        || mime.contains("bpmn")
        || mime.contains("pdf")
}

fn format_modified(secs: u64) -> String {
    // Simple: just show Unix timestamp as a human-readable relative string
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let diff = now.saturating_sub(secs);
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

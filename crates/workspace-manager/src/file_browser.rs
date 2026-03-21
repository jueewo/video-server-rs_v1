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
    pub is_viewable: bool, // images (png, jpg, svg, gif, webp, …)
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
    /// True if the workspace config has typed sub-folders under this path.
    pub has_typed_children: bool,
    /// True if this folder has a git_repo set in metadata.
    pub has_git_repo: bool,
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
                has_typed_children: false,
                has_git_repo: false,
            });
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            let size = metadata.len();
            let mime_type = friendly_mime(&name, &mime_guess::from_path(&name)
                .first_or_text_plain()
                .to_string());
            let is_editable = is_text_mime(&mime_type) || is_editable_by_extension(&name);
            let is_viewable = mime_type.starts_with("image/");
            let icon = file_icon_by_name(&name, &mime_type).to_string();
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
                is_viewable,
            });
        }
    }

    Ok(DirListing { folders, files })
}

/// Search files across the entire workspace by name substring (case-insensitive).
/// Returns up to `limit` matches sorted by relevance (exact name match first, then path match).
pub fn search_files(workspace_root: &Path, query: &str, limit: usize) -> Vec<FileEntry> {
    if !workspace_root.exists() || query.is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();

    let mut results: Vec<(u8, FileEntry)> = WalkDir::new(workspace_root)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden directories entirely
            !e.file_name().to_string_lossy().starts_with('.')
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();
            let rel_path = entry
                .path()
                .strip_prefix(workspace_root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            let path_lower = rel_path.to_lowercase();

            // Match against name or full path
            if !name_lower.contains(&query_lower) && !path_lower.contains(&query_lower) {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let mime_type = friendly_mime(
                &name,
                &mime_guess::from_path(&name)
                    .first_or_text_plain()
                    .to_string(),
            );
            let is_editable = is_text_mime(&mime_type) || is_editable_by_extension(&name);
            let is_viewable = mime_type.starts_with("image/");
            let icon = file_icon_by_name(&name, &mime_type).to_string();
            let size = metadata.len();
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

            // Relevance: 0 = exact name match, 1 = name contains, 2 = path-only match
            let rank = if name_lower == query_lower {
                0u8
            } else if name_lower.contains(&query_lower) {
                1u8
            } else {
                2u8
            };

            Some((
                rank,
                FileEntry {
                    name,
                    path: rel_path,
                    size,
                    size_str,
                    mime_type,
                    icon,
                    modified,
                    is_editable,
                    is_viewable,
                },
            ))
        })
        .collect();

    results.sort_by_key(|(rank, _)| *rank);
    results
        .into_iter()
        .take(limit)
        .map(|(_, f)| f)
        .collect()
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

            let mime_type = friendly_mime(&name, &mime_guess::from_path(&name)
                .first_or_text_plain()
                .to_string());
            let is_editable = is_text_mime(&mime_type) || is_editable_by_extension(&name);
            let is_viewable = mime_type.starts_with("image/");
            let icon = file_icon_by_name(&name, &mime_type).to_string();
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
                    is_viewable,
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


/// Check whether a directory recursively contains at least one file matching a type filter.
/// Bails early on first match for efficiency.
///
/// Filter values: "image", "video", "markdown", "diagram", "data".
pub fn folder_contains_type(folder: &Path, type_filter: &str) -> bool {
    WalkDir::new(folder)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .any(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            file_matches_type_filter(&name, type_filter)
        })
}

/// Check if a filename matches a type filter category.
pub fn file_matches_type_filter(name_lower: &str, type_filter: &str) -> bool {
    match type_filter {
        "image" => {
            name_lower.ends_with(".png") || name_lower.ends_with(".jpg")
                || name_lower.ends_with(".jpeg") || name_lower.ends_with(".gif")
                || name_lower.ends_with(".webp") || name_lower.ends_with(".bmp")
                || name_lower.ends_with(".ico")
        }
        "video" => {
            name_lower.ends_with(".mp4") || name_lower.ends_with(".webm")
                || name_lower.ends_with(".mov") || name_lower.ends_with(".avi")
                || name_lower.ends_with(".mkv")
        }
        "markdown" => {
            name_lower.ends_with(".md") || name_lower.ends_with(".mdx")
        }
        "diagram" => {
            name_lower.ends_with(".mmd") || name_lower.ends_with(".mermaid")
                || name_lower.ends_with(".drawio") || name_lower.ends_with(".excalidraw")
                || name_lower.ends_with(".bpmn") || name_lower.ends_with(".svg")
        }
        "data" => {
            name_lower.ends_with(".yaml") || name_lower.ends_with(".yml")
                || name_lower.ends_with(".json") || name_lower.ends_with(".csv")
                || name_lower.ends_with(".toml")
        }
        _ => true,
    }
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

pub fn file_icon_by_name(name: &str, mime: &str) -> &'static str {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "drawio" => return "workflow",
        "bpmn" => return "git-branch",
        _ => {}
    }
    file_icon(mime)
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
    } else if mime.contains("csv") {
        "table-2"
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

/// Human-readable display label for file types where mime_guess returns something
/// misleading (e.g. .mmd → karaoke format collision).
fn friendly_mime(name: &str, detected: &str) -> String {
    let lower = name.to_lowercase();
    if lower.ends_with(".mmd") || lower.ends_with(".mermaid") {
        return "Mermaid diagram".to_string();
    }
    if lower.ends_with(".excalidraw") {
        return "Excalidraw canvas".to_string();
    }
    if lower.ends_with(".drawio") {
        return "draw.io diagram".to_string();
    }
    detected.to_string()
}

/// Extensions that have dedicated editors regardless of what mime_guess says.
/// (e.g. .mmd is mis-identified as "application/vnd.chipnuts.karaoke-mmd")
fn is_editable_by_extension(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".mmd")
        || lower.ends_with(".mermaid")
        || lower.ends_with(".excalidraw")
        || lower.ends_with(".drawio")
}

/// A text file with its content, for LLM context gathering.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ContextFile {
    pub path: String,
    pub content: String,
    pub size: u64,
}

/// Collect text file contents from a directory (optionally recursive) for LLM context.
///
/// - Skips hidden files, binary files, and files larger than `max_file_size`.
/// - Stops collecting once `max_total_bytes` is reached.
/// - Returns files sorted alphabetically by path.
pub fn collect_context_files(
    workspace_root: &Path,
    subpath: &str,
    recursive: bool,
    max_file_size: u64,
    max_total_bytes: u64,
) -> Vec<ContextFile> {
    let dir = match safe_resolve(workspace_root, if subpath.is_empty() { "." } else { subpath }) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    // For workspace root (empty subpath), use workspace_root directly
    let dir = if subpath.is_empty() { workspace_root.to_path_buf() } else { dir };

    if !dir.exists() || !dir.is_dir() {
        return Vec::new();
    }

    let walker = WalkDir::new(&dir)
        .min_depth(1)
        .max_depth(if recursive { 10 } else { 1 })
        .into_iter()
        .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'));

    let mut entries: Vec<(String, PathBuf, u64)> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            let metadata = entry.metadata().ok()?;
            let size = metadata.len();

            // Skip large files
            if size > max_file_size || size == 0 {
                return None;
            }

            // Only include text-like files
            let mime = mime_guess::from_path(&name).first_or_text_plain().to_string();
            let mime_friendly = friendly_mime(&name, &mime);
            if !is_text_mime(&mime_friendly) && !is_editable_by_extension(&name) {
                return None;
            }

            let rel_path = entry
                .path()
                .strip_prefix(workspace_root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");

            Some((rel_path, entry.path().to_path_buf(), size))
        })
        .collect();

    // Sort alphabetically
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = Vec::new();
    let mut total_bytes: u64 = 0;

    for (rel_path, abs_path, size) in entries {
        if total_bytes + size > max_total_bytes {
            break;
        }

        if let Ok(content) = std::fs::read_to_string(&abs_path) {
            total_bytes += content.len() as u64;
            result.push(ContextFile {
                path: rel_path,
                content,
                size,
            });
        }
    }

    result
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

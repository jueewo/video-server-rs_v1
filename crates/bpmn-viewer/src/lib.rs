use askama::Template;
use async_trait::async_trait;
use axum::{http::StatusCode, response::{Html, IntoResponse, Response}};
use workspace_core::{FolderTypeRenderer, FolderViewContext};

#[derive(Template)]
#[template(path = "bpmn/view.html")]
pub struct BpmnViewerTemplate {
    pub authenticated: bool,
    pub page_title: String,
    pub title: String,
    pub slug: String,
    pub bpmn_xml: String,
    pub filename: String,
    pub created_at: String,
    pub is_owner: bool,
    /// URL to POST the saved XML to. Defaults to `/api/media/{slug}/save-bpmn`.
    pub save_url: String,
    /// URL for the back/cancel button. Defaults to `/media`.
    pub back_url: String,
    /// Label shown next to the back button and in the breadcrumb. Defaults to `"Media"`.
    pub back_label: String,
    /// Optional structured breadcrumb items. When non-empty, replaces the
    /// single back_label with individual clickable path segments.
    pub path_crumbs: Vec<(String, String)>,
}

impl BpmnViewerTemplate {
    pub fn new(
        authenticated: bool,
        title: String,
        slug: String,
        bpmn_xml: String,
        filename: String,
        created_at: String,
        is_owner: bool,
    ) -> Self {
        let save_url = format!("/api/media/{}/save-bpmn", slug);
        Self {
            authenticated,
            page_title: format!("BPMN: {}", title),
            title,
            slug,
            bpmn_xml,
            filename,
            created_at,
            is_owner,
            save_url,
            back_url: "/media".to_string(),
            back_label: "Media".to_string(),
            path_crumbs: vec![],
        }
    }
}

// ============================================================================
// Folder renderer
// ============================================================================

/// A single .bpmn file entry for the folder listing view.
pub struct BpmnFileEntry {
    pub name: String,
    /// Workspace-relative path, e.g. "processes/order-flow.bpmn"
    pub path: String,
    pub modified: String,
    /// First non-empty line from the sidecar `.md` file, if it exists.
    pub description: Option<String>,
    /// Workspace-relative path to the sidecar `.md` file (may or may not exist).
    pub md_path: String,
    /// Workspace-relative path to the sidecar `.bpmn.svg` file, if it exists.
    pub svg_path: Option<String>,
    /// Combined name + description text for client-side search filtering.
    pub search_text: String,
}

/// A group of `.bpmn` files, either top-level (name = None) or from a subfolder.
pub struct BpmnGroup {
    /// `None` means top-level / ungrouped; `Some(name)` is the subfolder name.
    pub name: Option<String>,
    /// Workspace-relative path to this group's directory (used for "New diagram here").
    pub path: String,
    pub files: Vec<BpmnFileEntry>,
}

#[derive(Template)]
#[template(path = "bpmn/folder.html")]
pub struct BpmnFolderTemplate {
    pub authenticated: bool,
    pub workspace_id: String,
    pub workspace_name: String,
    pub folder_path: String,
    pub folder_name: String,
    pub groups: Vec<BpmnGroup>,
    /// Workspace browse URL for the back link.
    pub back_url: String,
}

/// Folder-type renderer for `bpmn-simulator` folders.
///
/// Lists all `.bpmn` files in the folder (any depth) and renders them grouped
/// by their subfolder path as an inline view inside the workspace browser.
pub struct BpmnFolderRenderer;

#[async_trait]
impl FolderTypeRenderer for BpmnFolderRenderer {
    fn type_id(&self) -> &str {
        "bpmn-simulator"
    }

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        let folder_abs = ctx.workspace_root.join(&ctx.folder_path);

        // Collect .bpmn files at any depth, grouped by their relative subfolder path.
        let mut groups_map: std::collections::HashMap<Option<String>, Vec<BpmnFileEntry>> =
            std::collections::HashMap::new();

        for e in walkdir::WalkDir::new(&folder_abs)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        .map(|x| x.eq_ignore_ascii_case("bpmn"))
                        .unwrap_or(false)
            })
        {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e
                .path()
                .strip_prefix(&ctx.workspace_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| name.clone());

            // Group key: None for top-level files, relative subfolder path for everything else.
            let group_key: Option<String> = e
                .path()
                .parent()
                .and_then(|p| p.strip_prefix(&folder_abs).ok())
                .filter(|rel| !rel.as_os_str().is_empty())
                .map(|rel| rel.to_string_lossy().to_string());

            let modified = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| {
                    let secs = t
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()?
                        .as_secs();
                    let dt = time::OffsetDateTime::from_unix_timestamp(secs as i64).ok()?;
                    let fmt = time::format_description::parse("[year]-[month]-[day]").ok()?;
                    dt.format(&fmt).ok()
                })
                .unwrap_or_default();

            // Sidecar: same stem, .md extension
            let md_abs = e.path().with_extension("md");
            let md_path = md_abs
                .strip_prefix(&ctx.workspace_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    std::path::Path::new(&path)
                        .with_extension("md")
                        .to_string_lossy()
                        .to_string()
                });
            let description = std::fs::read_to_string(&md_abs).ok().and_then(|content| {
                content
                    .lines()
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.trim_start_matches('#').trim().to_string())
            });

            // SVG sidecar: same path + ".svg" (e.g. "foo.bpmn.svg")
            let svg_abs = std::path::PathBuf::from(format!("{}.svg", e.path().display()));
            let svg_path = if svg_abs.exists() {
                svg_abs
                    .strip_prefix(&ctx.workspace_root)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };

            let search_text = match &description {
                Some(d) => format!("{} {}", name, d),
                None => name.clone(),
            };

            groups_map.entry(group_key).or_default().push(BpmnFileEntry {
                name,
                path,
                modified,
                description,
                md_path,
                svg_path,
                search_text,
            });
        }

        // Build ordered groups: top-level first, then subfolders alphabetically
        let mut groups: Vec<BpmnGroup> = Vec::new();

        if let Some(mut top_files) = groups_map.remove(&None) {
            top_files.sort_by(|a, b| a.name.cmp(&b.name));
            groups.push(BpmnGroup {
                name: None,
                path: ctx.folder_path.clone(),
                files: top_files,
            });
        }

        let mut subfolder_groups: Vec<(String, Vec<BpmnFileEntry>)> = groups_map
            .into_iter()
            .filter_map(|(k, v)| k.map(|name| (name, v)))
            .collect();
        subfolder_groups.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (subfolder_name, mut files) in subfolder_groups {
            files.sort_by(|a, b| a.name.cmp(&b.name));
            let subfolder_path = if ctx.folder_path.is_empty() {
                subfolder_name.clone()
            } else {
                format!("{}/{}", ctx.folder_path, subfolder_name)
            };
            groups.push(BpmnGroup {
                name: Some(subfolder_name),
                path: subfolder_path,
                files,
            });
        }

        let back_url = format!("/workspaces/{}/browse", ctx.workspace_id);

        let template = BpmnFolderTemplate {
            authenticated: true,
            workspace_id: ctx.workspace_id,
            workspace_name: ctx.workspace_name,
            folder_path: ctx.folder_path,
            folder_name: ctx.folder_name,
            groups,
            back_url,
        };

        let html = template
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html).into_response())
    }
}

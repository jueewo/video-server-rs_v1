use crate::helpers::{check_scope, require_auth, verify_workspace_ownership, format_human_date, monaco_language, agent_format_helper_html, parent_browse_url, build_path_crumbs, count_files_in_dir};
use crate::{WorkspaceManagerState, WorkspaceConfig, WorkspaceDisplay, WorkspaceStats, WorkspaceListTemplate, NewWorkspaceTemplate, WorkspaceDashboardTemplate, WorkspaceBrowserTemplate, ImageViewerTemplate, DrawioEditorTemplate, MermaidEditorTemplate, ExcalidrawEditorTemplate, MarkdownPreviewTemplate, AgentViewerTemplate};
use crate::file_browser;
use crate::file_editor;
use crate::workspace_config;
use workspace_core::FolderViewContext;
use api_keys::middleware::AuthenticatedUser;
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use bpmn_viewer::BpmnViewerTemplate;
use docs_viewer::editor::EditorTemplate;
use pdf_viewer::PdfViewerTemplate;
use std::sync::Arc;
use tower_sessions::Session;
use crate::FileQuery;

pub(crate) async fn list_workspaces_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Ok(Redirect::to("/login").into_response());
    }

    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());
    let brand_name: String = session
        .get("brand_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    let rows = state.repo.list_user_workspaces(&user_id, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch all tags for user's workspaces in one query
    let mut tags_by_workspace = state.repo.get_workspace_tags_for_user(&user_id, &tenant_id)
        .await
        .unwrap_or_default();

    let workspaces: Vec<WorkspaceDisplay> = rows
        .into_iter()
        .map(|row| {
            let (workspace_id, name, description, created_at) =
                (row.workspace_id, row.name, row.description, row.created_at);
            let workspace_root = state.storage.workspace_root(&workspace_id);
            let file_count = count_files_in_dir(&workspace_root);
            let mut tags = tags_by_workspace.remove(&workspace_id).unwrap_or_default();
            tags.sort();
            WorkspaceDisplay {
                workspace_id,
                name,
                description: description.unwrap_or_default(),
                created_at: created_at.clone(),
                created_at_human: format_human_date(&created_at),
                file_count,
                total_size_str: String::new(),
                tags,
            }
        })
        .collect();

    // Collect all unique tags for the filter panel
    let mut all_tags: Vec<String> = workspaces
        .iter()
        .flat_map(|w| w.tags.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_tags.sort();

    let template = WorkspaceListTemplate {
        authenticated: true,
        workspaces,
        all_tags,
        brand_name,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html).into_response())
}

/// GET /workspaces/new — new workspace form
pub(crate) async fn new_workspace_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(_state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let template = NewWorkspaceTemplate { authenticated: true };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// GET /workspaces/{workspace_id} — workspace dashboard
pub(crate) async fn workspace_dashboard(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (name, description) =
        verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Single walkdir pass: compute file_count, total_size, and type breakdown.
    let mut file_count: i64 = 0;
    let mut total_size: u64 = 0;
    let mut image_count = 0usize;
    let mut video_count = 0usize;
    let mut doc_count = 0usize;
    let mut code_count = 0usize;
    let mut other_count = 0usize;

    if workspace_root.exists() {
        for entry in walkdir::WalkDir::new(&workspace_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            // Skip hidden files
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            file_count += 1;
            if let Ok(meta) = entry.metadata() {
                total_size += meta.len();
            }
            let mime = mime_guess::from_path(entry.path())
                .first_or_text_plain()
                .to_string();
            if mime.starts_with("image/") {
                image_count += 1;
            } else if mime.starts_with("video/") {
                video_count += 1;
            } else if mime.contains("pdf") || mime.contains("bpmn") || mime.contains("markdown") {
                doc_count += 1;
            } else if mime.starts_with("text/") || mime == "application/json"
                || mime == "application/yaml" || mime == "application/x-yaml"
            {
                code_count += 1;
            } else {
                other_count += 1;
            }
        }
    }

    let total_size_str = file_browser::format_size(total_size);
    let stats = WorkspaceStats { image_count, video_count, doc_count, code_count, other_count };

    // List top-level folders
    let mut folders = file_browser::list_dir(&workspace_root, "")
        .map(|e| e.folders)
        .unwrap_or_default();

    // Annotate folders with icon_url and type info from workspace.yaml + registry.
    let ws_config_opt = WorkspaceConfig::load(&workspace_root).ok();
    if let Some(ws_config) = ws_config_opt {
        let registry = state.folder_type_registry.read().unwrap();
        for folder in &mut folders {
            if let Some(fc) = ws_config.get_folder(&folder.path) {
                let type_id = fc.folder_type.as_str();
                if type_id != "default" {
                    if let Some(def) = registry.get_type(type_id) {
                        folder.folder_type = Some(type_id.to_string());
                        folder.type_color = def.color.clone();
                        folder.type_icon = Some(def.icon.clone());
                        folder.type_name = Some(def.name.clone());
                    } else {
                        folder.folder_type = Some(type_id.to_string());
                    }
                }
                // Check if folder has a git repo configured
                folder.has_git_repo = fc.metadata.get("git_repo")
                    .and_then(|v| v.as_str())
                    .map_or(false, |s| !s.is_empty());
            }
            // Check if any configured typed path lives under this folder
            if folder.folder_type.is_none() {
                let prefix = format!("{}/", folder.path);
                folder.has_typed_children = ws_config.folders.iter().any(|(path, fc)| {
                    path.starts_with(&prefix) && !fc.folder_type.is_default()
                });
            }
        }
    }
    for folder in &mut folders {
        let folder_abs = workspace_root.join(&folder.path);
        if file_browser::folder_has_icon(&folder_abs) {
            folder.icon_url = Some(format!(
                "/api/workspaces/{}/folder-icon/{}",
                workspace_id, folder.path
            ));
        }
    }

    // Gather recent files (up to 10, sorted by modification time)
    let recent_files = file_browser::recent_files(&workspace_root, 10);

    let row: Option<String> = state.repo.get_workspace_created_at(&workspace_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let created_at = row.unwrap_or_default();

    let workspace = WorkspaceDisplay {
        workspace_id: workspace_id.clone(),
        name,
        description: description.unwrap_or_default(),
        created_at: created_at.clone(),
        created_at_human: format_human_date(&created_at),
        file_count,
        total_size_str,
        tags: vec![],
    };

    let template = WorkspaceDashboardTemplate {
        authenticated: true,
        workspace,
        folders,
        recent_files,
        stats,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

#[derive(serde::Deserialize, Default)]
pub(crate) struct BrowseQuery {
    files: Option<String>,
}

/// GET /workspaces/{workspace_id}/browse/{*path}
#[allow(private_interfaces)]
pub(crate) async fn file_browser_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(path_parts): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Query(query): Query<BrowseQuery>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    let (workspace_id, subpath) = path_parts;
    file_browser_handler(workspace_id, subpath, session, state, query.files.as_deref() == Some("1")).await
}

/// GET /workspaces/{workspace_id}/browse
pub(crate) async fn file_browser_root_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Response, StatusCode> {
    check_scope(&user, "read")?;
    file_browser_handler(workspace_id, String::new(), session, state, false).await
}

async fn file_browser_handler(
    workspace_id: String,
    subpath: String,
    session: Session,
    state: Arc<WorkspaceManagerState>,
    force_files: bool,
) -> Result<Response, StatusCode> {
    let user_id = require_auth(&session).await?;
    let (workspace_name, workspace_description) =
        verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;
    let workspace_description = workspace_description.unwrap_or_default();

    let mut workspace_tags: Vec<String> = state.repo.get_workspace_tags(&workspace_id)
        .await
        .unwrap_or_default();
    workspace_tags.sort();

    let workspace_root = state.storage.workspace_root(&workspace_id);

    // Load workspace config once — used for renderer lookup and folder type annotation
    let ws_config_opt = WorkspaceConfig::load(&workspace_root).ok();

    // media-server folders redirect to the media library scoped to their vault
    if !force_files && !subpath.is_empty() {
        if let Some(ref ws_config) = ws_config_opt {
            if let Some(fc) = ws_config.get_folder(&subpath) {
                if fc.folder_type.as_str() == "media-server" {
                    if let Some(vault_id) = fc.metadata.get("vault_id").and_then(|v| v.as_str()) {
                        let redirect_url = format!("/media?vault_id={}", urlencoding::encode(vault_id));
                        return Ok(Redirect::to(&redirect_url).into_response());
                    }
                }
            }
        }
    }

    // Delegate to a registered renderer if one handles this folder type
    if !force_files && !subpath.is_empty() {
        if let Some(ref ws_config) = ws_config_opt {
            if let Some(fc) = ws_config.get_folder(&subpath) {
                let type_id = fc.folder_type.as_str();
                if let Some(renderer) = state.renderers.get(type_id) {
                    let folder_name = std::path::Path::new(&subpath)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&subpath)
                        .to_string();
                    let ctx = FolderViewContext {
                        workspace_id: workspace_id.clone(),
                        workspace_name,
                        folder_path: subpath,
                        folder_name,
                        user_id,
                        workspace_root,
                        metadata: fc.metadata.clone(),
                    };
                    return renderer.render_folder_view(ctx).await;
                }
            }
        }
    }

    let mut dir_listing =
        file_browser::list_dir(&workspace_root, &subpath).map_err(|_| StatusCode::NOT_FOUND)?;

    // Annotate folders with their type info from workspace.yaml + registry.
    // Also resolve the type of the current directory being browsed.
    let mut current_type_name: Option<String> = None;
    let mut current_type_color: Option<String> = None;
    let mut current_type_apps: Vec<(String, String)> = Vec::new();
    let mut current_type_id: Option<String> = None;
    let mut last_preview_url = String::new();

    if let Some(ws_config) = ws_config_opt {
        let registry = state.folder_type_registry.read().unwrap();

        // Current directory type + resolved app links
        if !subpath.is_empty() {
            if let Some(fc) = ws_config.get_folder(&subpath) {
                // Read preview URL from folder metadata
                if let Some(serde_yaml::Value::String(url)) = fc.metadata.get("last_preview_url") {
                    last_preview_url = url.clone();
                }
                let type_id = fc.folder_type.as_str();
                if type_id != "default" {
                    current_type_id = Some(type_id.to_string());
                    if let Some(def) = registry.get_type(type_id) {
                        current_type_name = Some(def.name.clone());
                        current_type_color = def.color.clone();
                        current_type_apps = def.apps.iter().map(|app| {
                            let url = app.url_template
                                .replace("{workspace_id}", &workspace_id)
                                .replace("{folder_path}", &subpath);
                            (app.label.clone(), url)
                        }).collect();
                    }
                }
            }
        }

        // Child folders
        for folder in &mut dir_listing.folders {
            if let Some(fc) = ws_config.get_folder(&folder.path) {
                let type_id = fc.folder_type.as_str();
                if type_id != "default" {
                    if let Some(def) = registry.get_type(type_id) {
                        folder.folder_type = Some(type_id.to_string());
                        folder.type_color = def.color.clone();
                        folder.type_icon = Some(def.icon.clone());
                        folder.type_name = Some(def.name.clone());
                    } else {
                        folder.folder_type = Some(type_id.to_string());
                    }
                }
                // Check if folder has a git repo configured
                folder.has_git_repo = fc.metadata.get("git_repo")
                    .and_then(|v| v.as_str())
                    .map_or(false, |s| !s.is_empty());
            }
            // Check if any configured typed path lives under this folder
            if folder.folder_type.is_none() {
                let prefix = format!("{}/", folder.path);
                folder.has_typed_children = ws_config.folders.iter().any(|(path, fc)| {
                    path.starts_with(&prefix) && !fc.folder_type.is_default()
                });
            }
        }
    }

    // Annotate icon_url for each folder that contains a thumbnail/icon image.
    for folder in &mut dir_listing.folders {
        let folder_abs = workspace_root.join(&folder.path);
        if file_browser::folder_has_icon(&folder_abs) {
            folder.icon_url = Some(format!(
                "/api/workspaces/{}/folder-icon/{}",
                workspace_id, folder.path
            ));
        }
    }

    // Annotate agent files with validation badges in agent-collection folders.
    // Only badge files that look like agent definitions (have an explicit role set).
    if current_type_id.as_deref() == Some("agent-collection") {
        for file in &mut dir_listing.files {
            let ext = file.name.rsplit('.').next().unwrap_or("");
            if matches!(ext, "md" | "yaml" | "yml" | "toml") {
                let file_abs = workspace_root.join(file.path.trim_start_matches('/'));
                if let Ok(agent) = agent_collection_processor::load_agent(&file_abs) {
                    // Skip files without an explicit role — they're not agent definitions
                    if agent.role != "assistant" {
                        if agent.active {
                            file.badge = Some(("valid".to_string(), "badge-success".to_string()));
                        } else {
                            file.badge = Some(("invalid".to_string(), "badge-error".to_string()));
                        }
                    }
                }
            }
        }
    }

    // Build breadcrumbs
    let mut breadcrumbs: Vec<(String, String)> = vec![(
        workspace_name.clone(),
        format!("/workspaces/{}/browse", workspace_id),
    )];

    if !subpath.is_empty() {
        let mut acc = String::new();
        for segment in subpath.split('/') {
            if segment.is_empty() {
                continue;
            }
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(segment);
            breadcrumbs.push((
                segment.to_string(),
                format!("/workspaces/{}/browse/{}", workspace_id, acc),
            ));
        }
    }

    let template = WorkspaceBrowserTemplate {
        authenticated: true,
        workspace_id,
        workspace_name,
        workspace_description,
        workspace_tags,
        current_path: subpath,
        breadcrumbs,
        folders: dir_listing.folders,
        files: dir_listing.files,
        current_type_name,
        current_type_color,
        current_type_apps,
        current_type_id,
        last_preview_url,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html).into_response())
}

/// GET /workspaces/{workspace_id}/edit-text?file=...
///
/// Opens a text file directly in Monaco editor (bypassing preview for markdown).
pub(crate) async fn edit_text_file_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<FileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) =
        verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let file_path = query.file.unwrap_or_default();
    if file_path.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_path)
        .to_string();

    let back_url = parent_browse_url(&workspace_id, &file_path);
    let encoded_path = urlencoding::encode(&file_path).into_owned();

    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Force Monaco editor for text files; treat missing files as empty (allows creating new docs)
    let content = file_editor::read_file(&workspace_root, &file_path).unwrap_or_default();
    let language = monaco_language(&ext);
    let save_url = format!(
        "/api/workspaces/{}/files/save-text?path={}",
        workspace_id, encoded_path
    );
    let cancel_url = back_url.clone();
    let mut template = EditorTemplate::new(
        true,
        workspace_id.clone(),
        file_name.clone(),
        content,
        file_name,
        language.to_string(),
        save_url,
        cancel_url,
    );
    template.back_url = back_url;
    template.back_label = workspace_name.clone();
    template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
    template.folder_path = std::path::Path::new(&file_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

    // Show format reference helper when editing agent files
    let folder_name = template.folder_path.trim_start_matches('/');
    let is_agent_folder = if !folder_name.is_empty() {
        workspace_config::WorkspaceConfig::load(&workspace_root)
            .ok()
            .and_then(|cfg| cfg.folders.get(folder_name).map(|f| f.folder_type.as_str() == "agent-collection"))
            .unwrap_or(false)
    } else {
        false
    };
    if is_agent_folder && matches!(ext.as_str(), "md" | "yaml" | "yml" | "toml") {
        template.helper_html = agent_format_helper_html(&ext);
    }

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

/// GET /workspaces/{workspace_id}/edit?file=...
///
/// Dispatches to the appropriate viewer/editor based on file extension:
/// - `.bpmn` → bpmn-viewer (view + edit)
/// - `.pdf`  → PDF.js viewer
/// - `.md`, `.markdown` → Markdown preview (with Edit button)
/// - other text files → Monaco editor
pub(crate) async fn open_file_page(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<FileQuery>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    let user_id = require_auth(&session).await?;
    let (workspace_name, _) =
        verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    let file_path = query.file.unwrap_or_default();
    if file_path.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_path)
        .to_string();

    let back_url = parent_browse_url(&workspace_id, &file_path);
    let encoded_path = urlencoding::encode(&file_path).into_owned();

    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let html = match ext.as_str() {
        "drawio" => {
            let fetch_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            DrawioEditorTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                file_name: file_name.clone(),
                fetch_url,
                save_url,
                back_url,
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "mmd" | "mermaid" => {
            let fetch_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            MermaidEditorTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                file_name: file_name.clone(),
                fetch_url,
                save_url,
                back_url,
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "excalidraw" => {
            let fetch_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            ExcalidrawEditorTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                file_name: file_name.clone(),
                fetch_url,
                save_url,
                back_url,
            }
            .render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "bpmn" => {
            let bpmn_xml = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let save_url = format!(
                "/api/workspaces/{}/bpmn/save?path={}",
                workspace_id, encoded_path
            );
            let mut template = BpmnViewerTemplate::new(
                true,
                file_name.clone(),
                workspace_id.clone(),
                bpmn_xml,
                file_name,
                String::new(),
                true, // is_owner — always true for workspace files
            );
            template.save_url = save_url;
            template.back_url = back_url;
            template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "pdf" => {
            let serve_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let mut template = PdfViewerTemplate::new(
                true,
                file_name.clone(),
                workspace_id.clone(),
                file_name,
                String::new(),
                None,
            );
            template.serve_url = serve_url;
            template.back_url = back_url;
            template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "md" | "markdown" | "mdx" => {
            // Markdown files → Preview mode with Edit button
            let raw_markdown = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let file_dir = std::path::Path::new(&file_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();

            // Check if this file is inside an agent-collection folder
            let is_agent_file = {
                let folder_name = file_dir.trim_start_matches('/');
                if !folder_name.is_empty() {
                    workspace_config::WorkspaceConfig::load(&workspace_root)
                        .ok()
                        .and_then(|cfg| cfg.folders.get(folder_name).map(|f| f.folder_type.as_str() == "agent-collection"))
                        .unwrap_or(false)
                } else {
                    false
                }
            };

            // Only use agent viewer if the file has an explicit role (not a plain README etc.)
            let agent_def = if is_agent_file && ext != "mdx" {
                agent_collection_processor::load_agent(
                    &workspace_root.join(file_path.trim_start_matches('/'))
                ).ok().filter(|a| a.role != "assistant")
            } else {
                None
            };

            if let Some(_) = &agent_def {

                let edit_url = format!(
                    "/workspaces/{}/edit-text?file={}",
                    workspace_id, encoded_path
                );

                let (agent_name, agent_role, agent_description, agent_model, agent_tools, agent_temperature, agent_folder_types, agent_autonomy, agent_max_iterations, agent_max_tokens, agent_timeout, agent_max_depth, agent_format, agent_active, agent_validation_errors, system_prompt) = match &agent_def {
                    Some(a) => (
                        a.name.clone(), a.role.clone(), a.description.clone(), a.model.clone(),
                        a.tools.clone(), a.temperature, a.folder_types.clone(), a.autonomy.clone(),
                        a.max_iterations, a.max_tokens, a.timeout, a.max_depth,
                        a.format.clone(), a.active, a.validation_errors.clone(), a.system_prompt.clone(),
                    ),
                    None => (
                        file_name.trim_end_matches(".md").trim_end_matches(".yaml").trim_end_matches(".yml").trim_end_matches(".toml").to_string(),
                        "unknown".to_string(), String::new(), "unknown".to_string(),
                        vec![], 1.0, vec![], "supervised".to_string(), 10, 4096, 300, 3,
                        ext.clone(), false,
                        vec![agent_collection_processor::ValidationError {
                            field: "file".to_string(),
                            message: "Failed to parse agent definition".to_string(),
                        }],
                        raw_markdown.clone(),
                    ),
                };

                let system_prompt_html = state.markdown_renderer.render_workspace(&system_prompt, &workspace_id, &file_dir);

                // Collect sibling agent files (.md and .yaml/.yml)
                let sibling_agents = {
                    let dir = workspace_root.join(file_dir.trim_start_matches('/'));
                    let current_name = std::path::Path::new(&file_path)
                        .file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let mut siblings: Vec<(String, String)> = std::fs::read_dir(&dir)
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_file())
                        .filter_map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            let is_agent_ext = name.ends_with(".md") || name.ends_with(".yaml") || name.ends_with(".yml") || name.ends_with(".toml");
                            if is_agent_ext && name != current_name {
                                // Only include files that are actual agent definitions
                                if let Ok(a) = agent_collection_processor::load_agent(&e.path()) {
                                    if a.role == "assistant" { return None; }
                                } else {
                                    return None;
                                }
                                let rel = if file_dir.is_empty() {
                                    name.clone()
                                } else {
                                    format!("{}/{}", file_dir.trim_start_matches('/'), name)
                                };
                                let display = name
                                    .trim_end_matches(".md")
                                    .trim_end_matches(".yaml")
                                    .trim_end_matches(".yml")
                                    .trim_end_matches(".toml")
                                    .to_string();
                                Some((display, rel))
                            } else {
                                None
                            }
                        })
                        .collect();
                    siblings.sort_by(|a, b| a.0.cmp(&b.0));
                    siblings
                };

                let template = AgentViewerTemplate {
                    authenticated: true,
                    workspace_id: workspace_id.clone(),
                    workspace_name: workspace_name.clone(),
                    agent_name,
                    agent_role,
                    agent_description,
                    agent_model,
                    agent_tools,
                    agent_temperature,
                    agent_folder_types,
                    agent_autonomy,
                    agent_max_iterations,
                    agent_max_tokens,
                    agent_timeout,
                    agent_max_depth,
                    agent_format,
                    agent_active,
                    agent_validation_errors,
                    system_prompt_html,
                    file_path: file_path.clone(),
                    raw_markdown,
                    edit_url,
                    back_url: back_url.clone(),
                    back_label: workspace_name.clone(),
                    sibling_agents,
                };
                template
                    .render()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            } else {
                // Standard markdown preview
                let render_input = if ext == "mdx" {
                    docs_viewer::markdown::preprocess_mdx(&raw_markdown)
                } else {
                    raw_markdown.clone()
                };
                let rendered_html = state.markdown_renderer.render_workspace(&render_input, &workspace_id, &file_dir);
                let edit_url = format!(
                    "/workspaces/{}/edit-text?file={}",
                    workspace_id, encoded_path
                );
                // Collect sibling .md/.mdx files from the same directory
                let sibling_docs = {
                    let dir = workspace_root.join(file_dir.trim_start_matches('/'));
                    let current_name = std::path::Path::new(&file_path)
                        .file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let mut siblings: Vec<(String, String)> = std::fs::read_dir(&dir)
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_file())
                        .filter_map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            if (name.ends_with(".md") || name.ends_with(".mdx")) && name != current_name {
                                let rel = if file_dir.is_empty() {
                                    name.clone()
                                } else {
                                    format!("{}/{}", file_dir.trim_start_matches('/'), name)
                                };
                                let display = name.clone();
                                Some((display, rel))
                            } else {
                                None
                            }
                        })
                        .collect();
                    siblings.sort_by(|a, b| a.0.cmp(&b.0));
                    siblings
                };
                let template = MarkdownPreviewTemplate {
                    authenticated: true,
                    workspace_id: workspace_id.clone(),
                    workspace_name: workspace_name.clone(),
                    title: file_name.clone(),
                    content: rendered_html,
                    file_path: file_path.clone(),
                    raw_markdown,
                    edit_url,
                    back_url: back_url.clone(),
                    back_label: workspace_name.clone(),
                    sibling_docs,
                };
                template
                    .render()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            }
        }
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "svg" | "ico" | "bmp" | "tiff"
        | "tif" => {
            let src_url = format!(
                "/api/workspaces/{}/files/serve?path={}",
                workspace_id, encoded_path
            );
            let file_size = {
                let abs = workspace_root.join(file_path.trim_start_matches('/'));
                abs.metadata()
                    .map(|m| file_browser::format_size(m.len()))
                    .unwrap_or_default()
            };
            let mime = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();
            let template = ImageViewerTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                workspace_name: workspace_name.clone(),
                title: file_name,
                src_url,
                back_url: back_url.clone(),
                back_label: workspace_name.clone(),
                mime_type: mime,
                file_size,
            };
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        "yaml" | "yml" | "toml" if {
            // Check if this YAML/TOML file is inside an agent-collection folder
            let folder_name = std::path::Path::new(&file_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .trim_start_matches('/');
            !folder_name.is_empty()
                && workspace_config::WorkspaceConfig::load(&workspace_root)
                    .ok()
                    .and_then(|cfg| cfg.folders.get(folder_name).map(|f| f.folder_type.as_str() == "agent-collection"))
                    .unwrap_or(false)
        } => {
            // Agent definition in YAML/TOML → render with agent viewer
            let raw_content = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let file_dir = std::path::Path::new(&file_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();

            let agent_def = agent_collection_processor::load_agent(
                &workspace_root.join(file_path.trim_start_matches('/'))
            ).ok();

            let edit_url = format!(
                "/workspaces/{}/edit-text?file={}",
                workspace_id, encoded_path
            );

            let (agent_name, agent_role, agent_description, agent_model, agent_tools, agent_temperature, agent_folder_types, agent_autonomy, agent_max_iterations, agent_max_tokens, agent_timeout, agent_max_depth, agent_format, agent_active, agent_validation_errors, system_prompt) = match &agent_def {
                Some(a) => (
                    a.name.clone(), a.role.clone(), a.description.clone(), a.model.clone(),
                    a.tools.clone(), a.temperature, a.folder_types.clone(), a.autonomy.clone(),
                    a.max_iterations, a.max_tokens, a.timeout, a.max_depth,
                    a.format.clone(), a.active, a.validation_errors.clone(), a.system_prompt.clone(),
                ),
                None => (
                    file_name.trim_end_matches(".yaml").trim_end_matches(".yml").trim_end_matches(".toml").to_string(),
                    "unknown".to_string(), String::new(), "unknown".to_string(),
                    vec![], 1.0, vec![], "supervised".to_string(), 10, 4096, 300, 3,
                    ext.clone(), false,
                    vec![agent_collection_processor::ValidationError {
                        field: "file".to_string(),
                        message: "Failed to parse agent definition".to_string(),
                    }],
                    raw_content.clone(),
                ),
            };

            let system_prompt_html = state.markdown_renderer.render_workspace(&system_prompt, &workspace_id, &file_dir);

            let sibling_agents = {
                let dir = workspace_root.join(file_dir.trim_start_matches('/'));
                let current_name = std::path::Path::new(&file_path)
                    .file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let mut siblings: Vec<(String, String)> = std::fs::read_dir(&dir)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        let is_agent_ext = name.ends_with(".md") || name.ends_with(".yaml") || name.ends_with(".yml") || name.ends_with(".toml");
                        if is_agent_ext && name != current_name {
                            // Only include files that are actual agent definitions
                            if let Ok(a) = agent_collection_processor::load_agent(&e.path()) {
                                if a.role == "assistant" { return None; }
                            } else {
                                return None;
                            }
                            let rel = if file_dir.is_empty() { name.clone() } else { format!("{}/{}", file_dir.trim_start_matches('/'), name) };
                            let display = name.trim_end_matches(".md").trim_end_matches(".yaml").trim_end_matches(".yml").trim_end_matches(".toml").to_string();
                            Some((display, rel))
                        } else { None }
                    })
                    .collect();
                siblings.sort_by(|a, b| a.0.cmp(&b.0));
                siblings
            };

            let template = AgentViewerTemplate {
                authenticated: true,
                workspace_id: workspace_id.clone(),
                workspace_name: workspace_name.clone(),
                agent_name, agent_role, agent_description, agent_model, agent_tools, agent_temperature,
                agent_folder_types, agent_autonomy, agent_max_iterations, agent_max_tokens,
                agent_timeout, agent_max_depth,
                agent_format, agent_active, agent_validation_errors, system_prompt_html,
                file_path: file_path.clone(),
                raw_markdown: raw_content,
                edit_url,
                back_url: back_url.clone(),
                back_label: workspace_name.clone(),
                sibling_agents,
            };
            template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        _ => {
            // Text-based files → Monaco editor
            let content = file_editor::read_file(&workspace_root, &file_path)
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let language = monaco_language(&ext);
            let save_url = format!(
                "/api/workspaces/{}/files/save-text?path={}",
                workspace_id, encoded_path
            );
            let cancel_url = back_url.clone();
            let mut template = EditorTemplate::new(
                true,
                workspace_id.clone(),
                file_name.clone(),
                content,
                file_name,
                language.to_string(),
                save_url,
                cancel_url,
            );
            template.back_url = back_url;
            template.path_crumbs = build_path_crumbs(&workspace_id, &workspace_name, &file_path);
            template
                .render()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    Ok(Html(html))
}

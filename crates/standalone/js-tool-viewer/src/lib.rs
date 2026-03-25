use askama::Template;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};
use tower_sessions::Session;

// ============================================================================
// State
// ============================================================================

#[derive(Clone)]
pub struct JsToolViewerState {
    pub pool: SqlitePool,
    /// Absolute path to the storage root (e.g. `./storage`).
    pub storage_base: PathBuf,
}

// ============================================================================
// Router
// ============================================================================

pub fn js_tool_viewer_routes(state: Arc<JsToolViewerState>) -> Router {
    Router::new()
        .route("/js-apps", get(gallery_handler))
        .route("/js-apps/{workspace_id}/{folder}", get(folder_gallery_handler))
        .route("/js-apps/{workspace_id}/{folder}/", get(folder_gallery_trailing_slash_handler))
        .route("/js-apps/{workspace_id}/{folder}/{*path}", get(serve_file_handler))
        .with_state(state)
}

// ============================================================================
// Auth helper
// ============================================================================

async fn require_auth(session: &Session) -> Result<String, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    session
        .get::<String>("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

// ============================================================================
// Ownership check helper
// ============================================================================

async fn check_workspace_ownership(
    pool: &SqlitePool,
    workspace_id: &str,
    user_id: &str,
) -> Result<(), StatusCode> {
    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM workspaces WHERE workspace_id = ?")
            .bind(workspace_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking workspace ownership: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    match owner {
        Some(owner_id) if owner_id == user_id => Ok(()),
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ============================================================================
// Minimal workspace.yaml parsing (no workspace-manager dep)
// ============================================================================

#[derive(Deserialize, Default)]
struct WorkspaceYamlPartial {
    #[serde(default)]
    folders: HashMap<String, FolderConfigPartial>,
}

#[derive(Deserialize)]
struct FolderConfigPartial {
    #[serde(rename = "type")]
    folder_type: String,
}

// ============================================================================
// Tool metadata
// ============================================================================

#[derive(Deserialize, Default)]
struct ToolMeta {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
}

async fn read_meta_yaml(tool_dir: &std::path::Path) -> (String, String) {
    let default_name = tool_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unnamed Tool")
        .to_string();

    let meta_path = tool_dir.join("meta.yaml");
    if let Ok(content) = tokio::fs::read_to_string(&meta_path).await {
        if let Ok(meta) = serde_yaml::from_str::<ToolMeta>(&content) {
            let title = if meta.title.is_empty() {
                default_name
            } else {
                meta.title
            };
            return (title, meta.description);
        }
    }
    (default_name, String::new())
}

/// Scan a folder directory for tool sub-dirs (each must have index.html).
async fn scan_tools_in_folder(
    workspace_root: &std::path::Path,
    workspace_id: &str,
    folder_path: &str,
) -> Vec<ToolEntry> {
    let folder_abs = workspace_root.join(folder_path);
    let mut rd = match tokio::fs::read_dir(&folder_abs).await {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut tools = Vec::new();
    while let Ok(Some(entry)) = rd.next_entry().await {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        if !entry_path.join("index.html").exists() {
            continue;
        }

        let tool_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if tool_name.is_empty() || tool_name.starts_with('.') {
            continue;
        }

        let (title, description) = read_meta_yaml(&entry_path).await;
        let url = format!("/js-apps/{}/{}/{}/", workspace_id, folder_path, tool_name);

        tools.push(ToolEntry {
            folder_path: folder_path.to_string(),
            title,
            description,
            url,
        });
    }
    tools
}

// ============================================================================
// Gallery handler — shows all js-tool folders across all user workspaces
// ============================================================================

struct ToolEntry {
    folder_path: String,
    title: String,
    description: String,
    url: String,
}

#[derive(Template)]
#[template(path = "js_tool/gallery.html")]
struct GalleryTemplate {
    tools: Vec<ToolEntry>,
    /// Optional heading: None = global gallery, Some(name) = folder gallery
    folder_name: Option<String>,
}

async fn gallery_handler(
    session: Session,
    State(state): State<Arc<JsToolViewerState>>,
) -> Result<Html<String>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // Find all workspaces owned by this user
    let workspaces: Vec<(String, String)> = sqlx::query_as(
        "SELECT workspace_id, name FROM workspaces WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query workspaces: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut tools: Vec<ToolEntry> = Vec::new();

    for (workspace_id, _workspace_name) in &workspaces {
        let workspace_root = state.storage_base.join("workspaces").join(workspace_id);
        let yaml_path = workspace_root.join("workspace.yaml");

        let config: WorkspaceYamlPartial = match tokio::fs::read_to_string(&yaml_path).await {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => continue,
        };

        for (folder_path, folder_config) in &config.folders {
            if !matches!(folder_config.folder_type.as_str(), "js-tool" | "web-app" | "runtime-app") {
                continue;
            }
            let mut folder_tools =
                scan_tools_in_folder(&workspace_root, workspace_id, folder_path).await;
            tools.append(&mut folder_tools);
        }
    }

    let template = GalleryTemplate { tools, folder_name: None };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

// ============================================================================
// Folder gallery handler — shows tools in one specific js-tool folder
// ============================================================================

async fn folder_gallery_handler(
    session: Session,
    Path((workspace_id, folder)): Path<(String, String)>,
    State(state): State<Arc<JsToolViewerState>>,
) -> Result<Response<Body>, StatusCode> {
    let user_id = require_auth(&session).await?;
    check_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage_base.join("workspaces").join(&workspace_id);
    let folder_abs = workspace_root.join(&folder);

    // If the folder itself has index.html at the root (single app, not a collection),
    // redirect to trailing-slash URL so relative fetches resolve correctly.
    if folder_abs.join("index.html").exists() {
        let redirect_url = format!("/js-apps/{}/{}/", workspace_id, folder);
        return Ok(Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::LOCATION, redirect_url)
            .body(Body::empty())
            .unwrap());
    }

    // Otherwise, scan for sub-tools (collection mode)
    let tools = scan_tools_in_folder(&workspace_root, &workspace_id, &folder).await;

    let template = GalleryTemplate {
        tools,
        folder_name: Some(folder.clone()),
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap())
}

// ============================================================================
// Trailing-slash handler — serves index.html for single-app folders
// ============================================================================

async fn folder_gallery_trailing_slash_handler(
    session: Session,
    Path((workspace_id, folder)): Path<(String, String)>,
    State(state): State<Arc<JsToolViewerState>>,
) -> Result<Response<Body>, StatusCode> {
    let user_id = require_auth(&session).await?;
    check_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    let workspace_root = state.storage_base.join("workspaces").join(&workspace_id);
    let folder_abs = workspace_root.join(&folder);
    let index_path = folder_abs.join("index.html");

    if index_path.exists() {
        let content = tokio::fs::read(&index_path)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(content))
            .unwrap());
    }

    Err(StatusCode::NOT_FOUND)
}

// ============================================================================
// File serve handler — serves individual tool files
// ============================================================================

async fn serve_file_handler(
    session: Session,
    Path((workspace_id, folder, path)): Path<(String, String, String)>,
    State(state): State<Arc<JsToolViewerState>>,
) -> Result<Response<Body>, StatusCode> {
    let user_id = require_auth(&session).await?;
    check_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    // Build target path: workspace_root / folder / path
    let workspace_root = state.storage_base.join("workspaces").join(&workspace_id);
    let target = workspace_root.join(&folder).join(&path);

    // Bare directory paths → serve index.html
    let target = if target.extension().is_none() || target.is_dir() {
        target.join("index.html")
    } else {
        target
    };

    // Path traversal check via canonicalize
    let canonical_target = match target.canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    let canonical_root = workspace_root
        .canonicalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !canonical_target.starts_with(&canonical_root) {
        tracing::warn!(
            "Path traversal attempt: workspace={}, folder={}, path={}",
            workspace_id,
            folder,
            path
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Read and serve
    let content = tokio::fs::read(&canonical_target)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mime = mime_for_path(&canonical_target);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(content))
        .unwrap())
}

fn mime_for_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json; charset=utf-8",
        Some("yaml") | Some("yml") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

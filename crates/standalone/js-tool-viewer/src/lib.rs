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
        .route("/js-apps/{workspace_id}/{*path}", get(serve_file_handler))
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

// ============================================================================
// Gallery handler
// ============================================================================

struct ToolEntry {
    workspace_name: String,
    folder_path: String,
    title: String,
    description: String,
    url: String,
}

#[derive(Template)]
#[template(path = "js_tool/gallery.html")]
struct GalleryTemplate {
    tools: Vec<ToolEntry>,
}

async fn gallery_handler(
    session: Session,
    State(state): State<Arc<JsToolViewerState>>,
) -> Result<Html<String>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // 1. Find all workspaces owned by this user
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

    for (workspace_id, workspace_name) in &workspaces {
        let workspace_root = state.storage_base.join("workspaces").join(workspace_id);
        let yaml_path = workspace_root.join("workspace.yaml");

        // 2. Parse workspace.yaml; skip if missing or unreadable
        let config: WorkspaceYamlPartial = match tokio::fs::read_to_string(&yaml_path).await {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => continue,
        };

        // 3. Find folders typed "js-tool"
        for (folder_path, folder_config) in &config.folders {
            if folder_config.folder_type != "js-tool" {
                continue;
            }

            let folder_abs = workspace_root.join(folder_path);
            let mut rd = match tokio::fs::read_dir(&folder_abs).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            // 4. Each sub-directory containing index.html is a tool
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

                if tool_name.is_empty() {
                    continue;
                }

                let (title, description) = read_meta_yaml(&entry_path).await;
                let url = format!("/js-apps/{}/{}/{}/", workspace_id, folder_path, tool_name);

                tools.push(ToolEntry {
                    workspace_name: workspace_name.clone(),
                    folder_path: folder_path.clone(),
                    title,
                    description,
                    url,
                });
            }
        }
    }

    let template = GalleryTemplate { tools };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

// ============================================================================
// File serve handler
// ============================================================================

async fn serve_file_handler(
    session: Session,
    Path((workspace_id, path)): Path<(String, String)>,
    State(state): State<Arc<JsToolViewerState>>,
) -> Result<Response<Body>, StatusCode> {
    let user_id = require_auth(&session).await?;

    // 1. Verify workspace ownership
    let owner: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM workspaces WHERE workspace_id = ?")
            .bind(&workspace_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking workspace ownership: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    match owner {
        Some(owner_id) if owner_id == user_id => {}
        Some(_) => return Err(StatusCode::FORBIDDEN),
        None => return Err(StatusCode::NOT_FOUND),
    }

    // 2. Build target path
    let workspace_root = state.storage_base.join("workspaces").join(&workspace_id);
    let target = workspace_root.join(&path);

    // Bare directory paths → serve index.html
    let target = if target.extension().is_none() || target.is_dir() {
        target.join("index.html")
    } else {
        target
    };

    // 3. Path traversal check via canonicalize
    let canonical_target = match target.canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    let canonical_root = workspace_root.canonicalize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !canonical_target.starts_with(&canonical_root) {
        tracing::warn!(
            "Path traversal attempt: workspace={}, path={}",
            workspace_id,
            path
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // 4. Read and serve
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

//! Preview route: serves template code + folder data by reference.
//!
//! URL: `/api/appstore/preview/{workspace_id}/{folder}/{*path}`
//!
//! Resolution order:
//! 1. Read app.yaml from workspace folder to get template id
//! 2. If requested path matches a template file → serve from template dir
//! 3. If requested path starts with `data/` → serve from workspace folder
//!    (stripping the `data/` prefix, mapping to the actual folder content)
//! 4. If path is empty or `/` → serve template's entry point (index.html)

use crate::AppstoreState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use std::sync::Arc;
use tokio::fs;
use tower_sessions::Session;

/// Serve a preview file for a template-based app in a workspace folder.
pub async fn preview_handler(
    session: Session,
    Path((workspace_id, folder, path)): Path<(String, String, String)>,
    State(state): State<Arc<AppstoreState>>,
) -> Result<Response, StatusCode> {
    // Auth check
    let user_id = require_auth(&session).await?;
    check_workspace_ownership(&state.pool, &workspace_id, &user_id).await?;

    // Resolve workspace folder
    let folder_abs = state
        .storage_base
        .join("workspaces")
        .join(&workspace_id)
        .join(&folder);

    if !folder_abs.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Read app.yaml to get template reference
    let app_config = crate::AppConfig::load(&folder_abs).map_err(|e| {
        tracing::error!("Failed to read app.yaml for preview: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let config = app_config.ok_or_else(|| {
        tracing::warn!("No app.yaml in {:?}", folder_abs);
        StatusCode::NOT_FOUND
    })?;

    let template = state.registry.get(&config.template).ok_or_else(|| {
        tracing::warn!("Template '{}' not found", config.template);
        StatusCode::NOT_FOUND
    })?;

    let template_dir = state.registry.template_dir(&config.template);

    // Data files: support JSON/YAML/TOML with auto-conversion
    if path.starts_with("data/") {
        let data_path = &path[5..]; // strip "data/"
        let target = folder_abs.join(data_path);

        // Try direct file first with path traversal check
        if let Ok(canonical) = target.canonicalize() {
            if let Ok(root) = folder_abs.canonicalize() {
                if canonical.starts_with(&root) && canonical.is_file() {
                    return serve_file(&canonical).await;
                }
            }
        }

        // Try format conversion (e.g. events.json -> events.yaml)
        if let Some((content, mime)) = crate::data_format::read_data_file(&target).await {
            return Ok(Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "no-cache")
                .body(Body::from(content))
                .unwrap());
        }

        return Err(StatusCode::NOT_FOUND);
    }

    // Determine which file to serve
    let resolved_path = if path.is_empty() || path == "/" {
        // Serve entry point from template
        template_dir.join(&template.entry)
    } else {
        // Try template dir first, fall back to folder data
        let tmpl_path = template_dir.join(&path);
        if tmpl_path.exists() {
            tmpl_path
        } else {
            // Maybe it's a data file referenced without the data/ prefix
            folder_abs.join(&path)
        }
    };

    // Path traversal protection
    let canonical = resolved_path.canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;
    let allowed_roots = [
        template_dir.canonicalize().ok(),
        folder_abs.canonicalize().ok(),
    ];
    let is_safe = allowed_roots
        .iter()
        .any(|root| root.as_ref().map_or(false, |r| canonical.starts_with(r)));

    if !is_safe {
        return Err(StatusCode::FORBIDDEN);
    }

    serve_file(&canonical).await
}

/// Serve the entry point (index.html) when no sub-path is given.
pub async fn preview_root_handler(
    session: Session,
    Path((workspace_id, folder)): Path<(String, String)>,
    State(state): State<Arc<AppstoreState>>,
) -> Result<Response, StatusCode> {
    preview_handler(
        session,
        Path((workspace_id, folder, String::new())),
        State(state),
    )
    .await
}

async fn serve_file(path: &std::path::Path) -> Result<Response, StatusCode> {
    if !path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let content = fs::read(path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let mime = mime_from_ext(path);

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(content))
        .unwrap())
}

fn mime_from_ext(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "yaml" | "yml" => "text/yaml; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        _ => "application/octet-stream",
    }
}

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

async fn check_workspace_ownership(
    pool: &sqlx::SqlitePool,
    workspace_id: &str,
    user_id: &str,
) -> Result<(), StatusCode> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT user_id FROM workspaces WHERE workspace_id = ?")
            .bind(workspace_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

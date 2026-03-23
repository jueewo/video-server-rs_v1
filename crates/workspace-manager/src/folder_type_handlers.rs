use crate::helpers::{check_scope, require_auth, verify_workspace_ownership};
use crate::{WorkspaceManagerState, FolderTypesTemplate, FolderTypeDefinition, InitTemplateRequest};
use crate::file_browser;
use api_keys::middleware::AuthenticatedUser;
use askama::Template;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    Extension,
};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{info, warn};

/// GET /folder-types — folder type registry management UI
pub(crate) async fn folder_types_page(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(_state): State<Arc<WorkspaceManagerState>>,
) -> Result<Html<String>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let template = FolderTypesTemplate { authenticated: true };
    let html = template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// GET /api/folder-types — list all registered folder types
pub(crate) async fn list_folder_types_handler(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let registry = state.folder_type_registry.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let types: Vec<&FolderTypeDefinition> = registry.list_types();
    let json = serde_json::to_value(&types).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json))
}

/// GET /api/folder-types/{id} — get a single folder type definition
pub(crate) async fn get_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(type_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "read")?;
    require_auth(&session).await?;

    let registry = state.folder_type_registry.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let def = registry.get_type(&type_id).ok_or(StatusCode::NOT_FOUND)?;
    let json = serde_json::to_value(def).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json))
}

/// POST /api/folder-types — create a new folder type
pub(crate) async fn create_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(def): Json<FolderTypeDefinition>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    check_scope(&user, "write")?;
    require_auth(&session).await?;

    if def.id.trim().is_empty() || def.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut registry = state.folder_type_registry.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    registry.create_type(def.clone()).map_err(|e| {
        warn!("Failed to create folder type: {}", e);
        StatusCode::CONFLICT
    })?;

    let json = serde_json::to_value(&def).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(json)))
}

/// PUT /api/folder-types/{id} — update an existing folder type
pub(crate) async fn update_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(type_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(def): Json<FolderTypeDefinition>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_scope(&user, "write")?;
    require_auth(&session).await?;

    let mut registry = state.folder_type_registry.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    registry.update_type(&type_id, def.clone()).map_err(|e| {
        warn!("Failed to update folder type '{}': {}", type_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let json = serde_json::to_value(&def).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json))
}

/// DELETE /api/folder-types/{id} — remove a folder type
pub(crate) async fn delete_folder_type_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(type_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    require_auth(&session).await?;

    let mut registry = state.folder_type_registry.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    registry.delete_type(&type_id).map_err(|e| {
        warn!("Failed to delete folder type '{}': {}", type_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/{workspace_id}/folder/init-template
///
/// Clones the git template associated with a folder type into the target folder.
pub(crate) async fn init_folder_from_template_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<InitTemplateRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Validate path (no traversal)
    let clean_path = request.path.trim_start_matches('/');
    for seg in clean_path.split('/') {
        if seg == ".." || seg == "." || seg.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Look up git_template from registry
    let git_url = {
        let registry = state.folder_type_registry.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let def = registry.get_type(&request.type_id).ok_or(StatusCode::NOT_FOUND)?;
        match &def.git_template {
            Some(url) if !url.is_empty() => url.clone(),
            _ => {
                warn!("Folder type '{}' has no git_template configured", request.type_id);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };

    let workspace_root = state.storage.workspace_root(&workspace_id);
    let target_dir = workspace_root.join(clean_path);

    // Refuse if directory already has content
    if target_dir.exists() {
        let is_empty = std::fs::read_dir(&target_dir)
            .map(|mut d| d.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            warn!("init-template: target directory {:?} is not empty", target_dir);
            return Err(StatusCode::CONFLICT);
        }
    }

    // Ensure parent exists
    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            warn!("Failed to create parent dir: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // Run git clone synchronously (blocks the handler — acceptable for a one-off admin op)
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg("--depth=1")
        .arg(&git_url)
        .arg(&target_dir)
        .output()
        .await
        .map_err(|e| {
            warn!("Failed to spawn git: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("git clone failed: {}", stderr);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!(
        "Cloned template {} into {:?} for workspace {}",
        git_url, target_dir, workspace_id
    );
    Ok(StatusCode::OK)
}

/// GET /api/workspaces/{workspace_id}/folder-icon/{*path}
/// Serves the thumbnail/icon image found in a workspace folder.
pub(crate) async fn serve_folder_icon_handler(
    user: Option<Extension<AuthenticatedUser>>,
    Path((workspace_id, folder_path)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Response {
    if check_scope(&user, "read").is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user_id = match require_auth(&session).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    if verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id)
        .await
        .is_err()
    {
        return StatusCode::FORBIDDEN.into_response();
    }

    let workspace_root = state.storage.workspace_root(&workspace_id);
    // Sanitize: reject path traversal
    let clean = folder_path.trim_start_matches('/');
    for seg in clean.split('/') {
        if seg == ".." || seg == "." {
            return StatusCode::BAD_REQUEST.into_response();
        }
    }
    let folder_abs = workspace_root.join(clean);

    match file_browser::icon_file_path(&folder_abs) {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(icon_path) => {
            let ext = icon_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let content_type = match ext {
                "svg" => "image/svg+xml",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                _ => "image/png",
            };
            match tokio::fs::read(&icon_path).await {
                Ok(bytes) => Response::builder()
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CACHE_CONTROL, "max-age=60")
                    .body(Body::from(bytes))
                    .unwrap(),
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

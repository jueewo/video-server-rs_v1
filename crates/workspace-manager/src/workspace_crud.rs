use crate::helpers::{check_scope, require_auth, verify_workspace_ownership};
use crate::{WorkspaceManagerState, CreateWorkspaceRequest, UpdateWorkspaceRequest, WorkspaceResponse, WorkspaceConfig};
use api_keys::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use common::storage::generate_workspace_id;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{info, warn};
use std::sync::Arc;

pub(crate) async fn create_workspace(
    user: Option<Extension<AuthenticatedUser>>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    let tenant_id: String = session
        .get("tenant_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "platform".to_string());

    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let workspace_id = generate_workspace_id();

    // Insert into DB
    state.repo.insert_workspace(
        &workspace_id,
        &user_id,
        &tenant_id,
        request.name.trim(),
        request.description.as_deref().filter(|s| !s.trim().is_empty()),
    )
    .await
    .map_err(|e| {
        warn!("Failed to insert workspace: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create workspace directory on filesystem
    state
        .storage
        .ensure_workspace_storage(&workspace_id)
        .map_err(|e| {
            warn!("Failed to create workspace directory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Write workspace.yaml using WorkspaceConfig
    let workspace_root = state.storage.workspace_root(&workspace_id);
    let config = WorkspaceConfig::new(
        request.name.clone(),
        request.description.clone().unwrap_or_default(),
    );
    config.save(&workspace_root).map_err(|e| {
        warn!("Failed to write workspace.yaml: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created workspace {} for user {}", workspace_id, user_id);

    Ok(Json(WorkspaceResponse {
        workspace_id,
        name: request.name,
        description: request.description,
        created_at: OffsetDateTime::now_utc().to_string(),
    }))
}

/// PUT /api/user/workspaces/{workspace_id} — update name/description
pub(crate) async fn update_workspace(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
    Json(request): Json<UpdateWorkspaceRequest>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    state.repo.update_workspace(
        &workspace_id,
        request.name.as_deref(),
        request.description.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Replace tags if provided
    if let Some(tags) = request.tags {
        state.repo.set_workspace_tags(&workspace_id, &tags)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    info!("Updated workspace {}", workspace_id);
    Ok(StatusCode::OK)
}

/// DELETE /api/user/workspaces/{workspace_id} — delete workspace
pub(crate) async fn delete_workspace(
    user: Option<Extension<AuthenticatedUser>>,
    Path(workspace_id): Path<String>,
    session: Session,
    State(state): State<Arc<WorkspaceManagerState>>,
) -> Result<StatusCode, StatusCode> {
    check_scope(&user, "write")?;
    let user_id = require_auth(&session).await?;
    verify_workspace_ownership(state.repo.as_ref(), &workspace_id, &user_id).await?;

    // Delete DB record
    state.repo.delete_workspace(&workspace_id, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Remove filesystem directory
    let workspace_root = state.storage.workspace_root(&workspace_id);
    if workspace_root.exists() {
        std::fs::remove_dir_all(&workspace_root).map_err(|e| {
            warn!("Failed to delete workspace directory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    info!("Deleted workspace {} for user {}", workspace_id, user_id);
    Ok(StatusCode::NO_CONTENT)
}

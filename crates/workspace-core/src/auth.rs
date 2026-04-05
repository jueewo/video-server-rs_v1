//! Shared auth helpers for workspace handler crates.
//!
//! These are extracted here so that crates which provide workspace routes
//! (site-overview, agent-registry, etc.) can share auth logic without
//! depending on workspace-manager.

use api_keys::middleware::{require_scope, AuthenticatedUser};
use axum::{extract::Extension, http::StatusCode};
use db::workspaces::WorkspaceRepository;
use tower_sessions::Session;

/// Get authenticated user_id from session, or return 401/500.
pub async fn require_auth(session: &Session) -> Result<String, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(user_id)
}

/// Platform-admin guard: only the user whose id matches PLATFORM_ADMIN_ID env var.
pub async fn require_platform_admin(session: &Session) -> Result<String, StatusCode> {
    let user_id = require_auth(session).await?;
    let admin_id = std::env::var("PLATFORM_ADMIN_ID")
        .unwrap_or_else(|_| "7bda815e-729a-49ea-88c5-3ca59b9ce487".to_string());
    if user_id != admin_id {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(user_id)
}

/// Verify that `workspace_id` belongs to `user_id`. Returns (name, description).
pub async fn verify_workspace_ownership(
    repo: &dyn WorkspaceRepository,
    workspace_id: &str,
    user_id: &str,
) -> Result<(String, Option<String>), StatusCode> {
    repo.verify_workspace_ownership(workspace_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

/// Check API key scope if authenticated via API key (session auth has full permissions).
pub fn check_scope(
    user_ext: &Option<Extension<AuthenticatedUser>>,
    scope: &str,
) -> Result<(), StatusCode> {
    if let Some(Extension(user)) = user_ext {
        require_scope(user, scope)?;
    }
    Ok(())
}

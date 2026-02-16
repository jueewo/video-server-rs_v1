use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{delete, get, post, put},
    Router,
};
use common::storage::{generate_vault_id, UserStorageManager};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{info, warn};

#[derive(Clone)]
pub struct VaultManagerState {
    pub pool: SqlitePool,
    pub storage: Arc<UserStorageManager>,
}

impl VaultManagerState {
    pub fn new(pool: SqlitePool, storage: Arc<UserStorageManager>) -> Self {
        Self {
            pool,
            storage,
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVaultRequest {
    pub name: String,
    pub vault_code: Option<String>, // Optional custom vault code
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVaultRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultResponse {
    pub vault_id: String,
    pub vault_code: Option<String>,
    pub name: String,
    pub is_default: bool,
    pub created_at: String,
    pub media_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultListResponse {
    pub vaults: Vec<VaultResponse>,
}

// ============================================================================
// Template Types
// ============================================================================

#[derive(Template, Clone)]
#[template(path = "vaults/list.html")]
pub struct VaultListTemplate {
    pub authenticated: bool,
    pub vaults: Vec<VaultDisplay>,
}

#[derive(Template)]
#[template(path = "vaults/new.html")]
pub struct NewVaultTemplate {
    pub authenticated: bool,
}

#[derive(Clone)]
pub struct VaultDisplay {
    pub vault_id: String,
    pub vault_code: String,
    pub name: String,
    pub is_default: bool,
    pub created_at: String,
    pub created_at_human: String,
    pub media_count: i64,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_human_date(date_str: &str) -> String {
    if let Ok(dt) = OffsetDateTime::parse(
        date_str,
        &time::format_description::well_known::Iso8601::DEFAULT,
    ) {
        let now = OffsetDateTime::now_utc();
        let diff = now - dt;

        let days = diff.whole_days();
        if days == 0 {
            "Today".to_string()
        } else if days == 1 {
            "Yesterday".to_string()
        } else if days < 7 {
            format!("{} days ago", days)
        } else if days < 30 {
            format!("{} weeks ago", days / 7)
        } else if days < 365 {
            format!("{} months ago", days / 30)
        } else {
            format!("{} years ago", days / 365)
        }
    } else {
        date_str.to_string()
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// Create a new vault
#[tracing::instrument(skip(session, state, request))]
pub async fn create_vault(
    session: Session,
    State(state): State<Arc<VaultManagerState>>,
    Json(request): Json<CreateVaultRequest>,
) -> Result<Json<VaultResponse>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!("Unauthenticated attempt to create vault");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate name
    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Generate or use provided vault_code
    let vault_code = if let Some(code) = request.vault_code {
        // Validate custom code
        if !code.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            warn!("Invalid vault code format: {}", code);
            return Err(StatusCode::BAD_REQUEST);
        }
        code
    } else {
        // Generate random vault ID
        generate_vault_id()
    };

    // Check if vault_id already exists
    let existing: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM storage_vaults WHERE vault_id = ?"
    )
    .bind(&vault_code)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        warn!("Vault code already exists: {}", vault_code);
        return Err(StatusCode::CONFLICT);
    }

    // Check if this is the user's first vault (make it default)
    let vault_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM storage_vaults WHERE user_id = ?"
    )
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_default = vault_count == 0;

    // Insert vault into database
    sqlx::query(
        "INSERT INTO storage_vaults (vault_id, user_id, vault_name, is_default, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&vault_code)
    .bind(&user_id)
    .bind(&request.name)
    .bind(if is_default { 1 } else { 0 })
    .bind(OffsetDateTime::now_utc().format(&time::format_description::well_known::Iso8601::DEFAULT).unwrap())
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create vault directories on filesystem
    state
        .storage
        .ensure_vault_storage(&vault_code)
        .map_err(|e| {
            warn!("Failed to create vault directories: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        "Created vault {} for user {}",
        vault_code, user_id
    );

    Ok(Json(VaultResponse {
        vault_id: vault_code.clone(),
        vault_code: Some(vault_code),
        name: request.name,
        is_default,
        created_at: OffsetDateTime::now_utc().to_string(),
        media_count: 0,
    }))
}

/// List all vaults for the authenticated user
#[tracing::instrument(skip(session, state))]
pub async fn list_vaults(
    session: Session,
    State(state): State<Arc<VaultManagerState>>,
) -> Result<Json<VaultListResponse>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get vaults with media count
    let vaults: Vec<(String, String, i32, String)> = sqlx::query_as(
        r#"
        SELECT
            v.vault_id,
            v.vault_name,
            v.is_default,
            v.created_at
        FROM storage_vaults v
        WHERE v.user_id = ?
        ORDER BY v.is_default DESC, v.created_at ASC
        "#
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut vault_responses = Vec::new();

    for (vault_id, name, is_default, created_at) in vaults {
        // Count media in this vault
        let media_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM media_items WHERE vault_id = ?"
        )
        .bind(&vault_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        vault_responses.push(VaultResponse {
            vault_id: vault_id.clone(),
            vault_code: Some(vault_id),
            name,
            is_default: is_default == 1,
            created_at,
            media_count,
        });
    }

    Ok(Json(VaultListResponse {
        vaults: vault_responses,
    }))
}

/// Update a vault's name
#[tracing::instrument(skip(session, state, request))]
pub async fn update_vault(
    Path(vault_id): Path<String>,
    session: Session,
    State(state): State<Arc<VaultManagerState>>,
    Json(request): Json<UpdateVaultRequest>,
) -> Result<StatusCode, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify ownership
    let owner: Option<String> = sqlx::query_scalar(
        "SELECT user_id FROM storage_vaults WHERE vault_id = ?"
    )
    .bind(&vault_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if owner.as_ref() != Some(&user_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Update name if provided
    if let Some(name) = request.name {
        if name.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        sqlx::query("UPDATE storage_vaults SET vault_name = ?, updated_at = ? WHERE vault_id = ?")
            .bind(&name)
            .bind(OffsetDateTime::now_utc().format(&time::format_description::well_known::Iso8601::DEFAULT).unwrap())
            .bind(&vault_id)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        info!("Updated vault {} name to '{}'", vault_id, name);
    }

    Ok(StatusCode::OK)
}

/// Set a vault as default
#[tracing::instrument(skip(session, state))]
pub async fn set_default_vault(
    Path(vault_id): Path<String>,
    session: Session,
    State(state): State<Arc<VaultManagerState>>,
) -> Result<StatusCode, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify ownership
    let owner: Option<String> = sqlx::query_scalar(
        "SELECT user_id FROM storage_vaults WHERE vault_id = ?"
    )
    .bind(&vault_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if owner.as_ref() != Some(&user_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Start transaction
    let mut tx = state.pool.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Unset all default vaults for this user
    sqlx::query("UPDATE storage_vaults SET is_default = 0 WHERE user_id = ?")
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Set this vault as default
    sqlx::query("UPDATE storage_vaults SET is_default = 1, updated_at = ? WHERE vault_id = ?")
        .bind(OffsetDateTime::now_utc().format(&time::format_description::well_known::Iso8601::DEFAULT).unwrap())
        .bind(&vault_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Set vault {} as default for user {}", vault_id, user_id);

    Ok(StatusCode::OK)
}

/// Delete a vault (only if it has no media)
#[tracing::instrument(skip(session, state))]
pub async fn delete_vault(
    Path(vault_id): Path<String>,
    session: Session,
    State(state): State<Arc<VaultManagerState>>,
) -> Result<StatusCode, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify ownership
    let owner: Option<String> = sqlx::query_scalar(
        "SELECT user_id FROM storage_vaults WHERE vault_id = ?"
    )
    .bind(&vault_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if owner.as_ref() != Some(&user_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if vault has media
    let media_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM media_items WHERE vault_id = ?"
    )
    .bind(&vault_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if media_count > 0 {
        warn!("Cannot delete vault {} - contains {} media items", vault_id, media_count);
        return Err(StatusCode::CONFLICT);
    }

    // Delete vault
    let rows_affected = sqlx::query("DELETE FROM storage_vaults WHERE vault_id = ? AND user_id = ?")
        .bind(&vault_id)
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .rows_affected();

    if rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    info!("Deleted vault {} for user {}", vault_id, user_id);

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// UI Handlers
// ============================================================================

/// List vaults page
#[tracing::instrument(skip(session, state))]
pub async fn list_vaults_page(
    session: Session,
    State(state): State<Arc<VaultManagerState>>,
) -> Result<Html<String>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get vaults
    let vaults: Vec<(String, String, i32, String)> = sqlx::query_as(
        "SELECT vault_id, vault_name, is_default, created_at FROM storage_vaults WHERE user_id = ? ORDER BY is_default DESC, created_at ASC"
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut vault_displays = Vec::new();

    for (vault_id, name, is_default, created_at) in vaults {
        // Count media in this vault
        let media_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM media_items WHERE vault_id = ?"
        )
        .bind(&vault_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

        vault_displays.push(VaultDisplay {
            vault_id: vault_id.clone(),
            vault_code: vault_id,
            name,
            is_default: is_default == 1,
            created_at: created_at.clone(),
            created_at_human: format_human_date(&created_at),
            media_count,
        });
    }

    let template = VaultListTemplate {
        authenticated: true,
        vaults: vault_displays,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// New vault page
#[tracing::instrument(skip(session, _state))]
pub async fn new_vault_page(
    session: Session,
    State(_state): State<Arc<VaultManagerState>>,
) -> Result<Html<String>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let template = NewVaultTemplate {
        authenticated: true,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

// ============================================================================
// Router
// ============================================================================

pub fn vault_routes(state: Arc<VaultManagerState>) -> Router {
    Router::new()
        // API routes
        .route("/api/user/vaults", post(create_vault))
        // Note: GET /api/user/vaults is handled by media-hub for upload form compatibility
        .route("/api/user/vaults/:vault_id", put(update_vault))
        .route("/api/user/vaults/:vault_id", delete(delete_vault))
        .route("/api/user/vaults/:vault_id/set-default", post(set_default_vault))
        // UI routes
        .route("/vaults", get(list_vaults_page))
        .route("/vaults/new", get(new_vault_page))
        .with_state(state)
}

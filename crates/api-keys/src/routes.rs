use crate::{db, CreateApiKeyRequest, UpdateApiKeyRequest};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{error, warn};

// -------------------------------
// Templates
// -------------------------------

#[derive(Template)]
#[template(path = "api-keys/list.html")]
struct ApiKeysListTemplate {
    authenticated: bool,
    user_id: String,
    api_keys: Vec<crate::ApiKey>,
}

#[derive(Template)]
#[template(path = "api-keys/create.html")]
struct CreateApiKeyTemplate {
    authenticated: bool,
    user_id: String,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "api-keys/created.html")]
struct ApiKeyCreatedTemplate {
    authenticated: bool,
    user_id: String,
    key: String,
    key_prefix: String,
    name: String,
}

// -------------------------------
// Router Setup
// -------------------------------

pub fn api_key_routes(pool: Arc<SqlitePool>) -> Router {
    Router::new()
        // UI Routes (session auth required)
        .route("/profile/api-keys", get(list_api_keys_page_handler))
        .route("/profile/api-keys/create", get(create_api_key_page_handler))
        .route(
            "/profile/api-keys/create",
            post(create_api_key_form_handler),
        )
        .route(
            "/profile/api-keys/:id/revoke",
            post(revoke_api_key_handler),
        )
        // API Routes (JSON, session auth required)
        .route("/api/user/api-keys", post(create_api_key_json_handler))
        .route("/api/user/api-keys", get(list_api_keys_json_handler))
        .route("/api/user/api-keys/:id", get(get_api_key_json_handler))
        .route("/api/user/api-keys/:id", axum::routing::put(update_api_key_json_handler))
        .route("/api/user/api-keys/:id", axum::routing::delete(delete_api_key_json_handler))
        .with_state(pool)
}

// -------------------------------
// Helper Functions
// -------------------------------

async fn get_user_id_from_session(session: &Session) -> Result<String, Response> {
    let authenticated = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }

    session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())
}

// -------------------------------
// UI Handlers
// -------------------------------

/// List all API keys (UI page)
async fn list_api_keys_page_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let api_keys = db::list_user_api_keys(&pool, &user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to list API keys");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?;

    let template = ApiKeysListTemplate {
        authenticated: true,
        user_id,
        api_keys,
    };

    Ok(Html(template.render().unwrap()))
}

/// Show create API key form
async fn create_api_key_page_handler(
    session: Session,
) -> Result<Html<String>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    let template = CreateApiKeyTemplate {
        authenticated: true,
        user_id,
        error: None,
    };

    Ok(Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyFormData {
    name: String,
    description: Option<String>,
    scopes: Option<String>, // Comma-separated or checkboxes as "scope1,scope2"
    expiration: Option<String>, // "never", "30days", "90days", "1year", "custom"
    custom_expiration: Option<String>, // ISO 8601 date
}

/// Handle create API key form submission
async fn create_api_key_form_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
    Form(form): Form<CreateApiKeyFormData>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    // Parse scopes
    let scopes: Vec<String> = match &form.scopes {
        Some(scopes_str) if !scopes_str.is_empty() => {
            scopes_str.split(',').map(|s| s.trim().to_string()).collect()
        }
        _ => vec!["read".to_string()], // Default to read-only
    };

    // Validate scopes
    if let Err(e) = crate::validate_scopes(&scopes) {
        let template = CreateApiKeyTemplate {
            authenticated: true,
            user_id,
            error: Some(e.to_string()),
        };
        return Ok(Html(template.render().unwrap()).into_response());
    }

    // Parse expiration
    let expires_at = match form.expiration.as_deref() {
        Some("never") | None => None,
        Some("30days") => Some(
            (chrono::Utc::now() + chrono::Duration::days(30))
                .to_rfc3339()
        ),
        Some("90days") => Some(
            (chrono::Utc::now() + chrono::Duration::days(90))
                .to_rfc3339()
        ),
        Some("1year") => Some(
            (chrono::Utc::now() + chrono::Duration::days(365))
                .to_rfc3339()
        ),
        Some("custom") => form.custom_expiration,
        _ => None,
    };

    let request = CreateApiKeyRequest {
        name: form.name,
        description: form.description,
        scopes,
        expires_at,
    };

    match db::create_api_key(&pool, &user_id, request).await {
        Ok(response) => {
            // Show the created key (only once!)
            let template = ApiKeyCreatedTemplate {
                authenticated: true,
                user_id,
                key: response.key,
                key_prefix: response.api_key.key_prefix,
                name: response.api_key.name,
            };
            Ok(Html(template.render().unwrap()).into_response())
        }
        Err(e) => {
            error!(error = %e, "Failed to create API key");
            let template = CreateApiKeyTemplate {
                authenticated: true,
                user_id,
                error: Some(format!("Failed to create API key: {}", e)),
            };
            Ok(Html(template.render().unwrap()).into_response())
        }
    }
}

/// Revoke an API key (form submission)
async fn revoke_api_key_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::revoke_api_key(&pool, id, &user_id).await {
        Ok(true) => Ok(Redirect::to("/profile/api-keys").into_response()),
        Ok(false) => {
            warn!("API key not found or not owned by user");
            Err(StatusCode::NOT_FOUND.into_response())
        }
        Err(e) => {
            error!(error = %e, "Failed to revoke API key");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// -------------------------------
// JSON API Handlers
// -------------------------------

/// Create API key (JSON API)
async fn create_api_key_json_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<crate::ApiKeyResponse>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::create_api_key(&pool, &user_id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!(error = %e, "Failed to create API key");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

/// List API keys (JSON API)
async fn list_api_keys_json_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
) -> Result<Json<Vec<crate::ApiKey>>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::list_user_api_keys(&pool, &user_id).await {
        Ok(keys) => Ok(Json(keys)),
        Err(e) => {
            error!(error = %e, "Failed to list API keys");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

/// Get specific API key (JSON API)
async fn get_api_key_json_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
    Path(id): Path<i32>,
) -> Result<Json<crate::ApiKey>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::get_api_key_by_id(&pool, id, &user_id).await {
        Ok(Some(key)) => Ok(Json(key)),
        Ok(None) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to get API key");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

/// Update API key metadata (JSON API)
async fn update_api_key_json_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
    Path(id): Path<i32>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<Json<crate::ApiKey>, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::update_api_key(&pool, id, &user_id, request).await {
        Ok(Some(key)) => Ok(Json(key)),
        Ok(None) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to update API key");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

/// Delete (revoke) API key (JSON API)
async fn delete_api_key_json_handler(
    State(pool): State<Arc<SqlitePool>>,
    session: Session,
    Path(id): Path<i32>,
) -> Result<StatusCode, Response> {
    let user_id = get_user_id_from_session(&session).await?;

    match db::revoke_api_key(&pool, id, &user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND.into_response()),
        Err(e) => {
            error!(error = %e, "Failed to revoke API key");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{self, info, warn};

#[derive(Clone)]
pub struct AccessCodeState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct CreateAccessCodeRequest {
    pub code: String,
    pub description: Option<String>,
    pub expires_at: Option<String>, // ISO 8601 datetime string
    pub media_items: Vec<MediaItem>,
}

#[derive(Deserialize, Serialize)]
pub struct MediaItem {
    pub media_type: String, // "video" or "image"
    pub media_slug: String,
}

#[derive(Serialize)]
pub struct AccessCodeResponse {
    pub id: i32,
    pub code: String,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub media_items: Vec<MediaItem>,
}

#[derive(Serialize)]
pub struct AccessCodeListResponse {
    pub access_codes: Vec<AccessCodeResponse>,
}

#[derive(Serialize)]
pub struct MediaResource {
    pub media_type: String,
    pub slug: String,
    pub title: String,
}

#[tracing::instrument(skip(session, state, request))]
pub async fn create_access_code(
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
    Json(request): Json<CreateAccessCodeRequest>,
) -> Result<Json<AccessCodeResponse>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!(
            event = "access_denied",
            resource = "access_codes",
            action = "create",
            reason = "unauthenticated",
            "Unauthenticated attempt to create access code"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session for ownership validation
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    // Validate code format
    if request.code.is_empty() || request.code.len() > 50 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if code already exists
    let existing: Option<i32> = sqlx::query_scalar("SELECT id FROM access_codes WHERE code = ?")
        .bind(&request.code)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        warn!(
            event = "access_code_conflict",
            code = %request.code,
            user_id = %user_id,
            "Attempted to create duplicate access code"
        );
        return Err(StatusCode::CONFLICT);
    }

    // Parse expiration date
    let expires_at = if let Some(ref expiry_str) = request.expires_at {
        Some(
            OffsetDateTime::parse(
                expiry_str,
                &time::format_description::well_known::Iso8601::DEFAULT,
            )
            .map_err(|_| StatusCode::BAD_REQUEST)?,
        )
    } else {
        None
    };

    // Insert access code
    let code_id: i32 = sqlx::query_scalar(
        "INSERT INTO access_codes (code, description, expires_at, created_by) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(&request.code)
    .bind(&request.description)
    .bind(expires_at.map(|dt| {
        dt.format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap()
    }))
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert permissions (only for owned media)
    for item in &request.media_items {
        if item.media_type != "video" && item.media_type != "image" {
            warn!(
                event = "invalid_request",
                media_type = %item.media_type,
                "Invalid media type in access code request"
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate ownership
        let is_owner = match item.media_type.as_str() {
            "video" => {
                let owner: Option<String> =
                    sqlx::query_scalar("SELECT user_id FROM videos WHERE slug = ?")
                        .bind(&item.media_slug)
                        .fetch_optional(&state.pool)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                owner.as_ref() == Some(&user_id)
            }
            "image" => {
                let owner: Option<String> =
                    sqlx::query_scalar("SELECT user_id FROM images WHERE slug = ?")
                        .bind(&item.media_slug)
                        .fetch_optional(&state.pool)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                owner.as_ref() == Some(&user_id)
            }
            _ => false,
        };

        if !is_owner {
            warn!(
                event = "access_denied",
                resource = "media",
                action = "share",
                user_id = %user_id,
                media_type = %item.media_type,
                media_slug = %item.media_slug,
                reason = "not_owner",
                "User attempted to share media they don't own"
            );
            return Err(StatusCode::FORBIDDEN);
        }

        sqlx::query(
            "INSERT INTO access_code_permissions (access_code_id, media_type, media_slug) VALUES (?, ?, ?)"
        )
        .bind(code_id)
        .bind(&item.media_type)
        .bind(&item.media_slug)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return created access code
    info!(
        event = "access_code_created",
        code = %request.code,
        user_id = %user_id,
        media_count = request.media_items.len(),
        "Access code created successfully"
    );

    Ok(Json(AccessCodeResponse {
        id: code_id,
        code: request.code,
        description: request.description,
        expires_at: request.expires_at,
        created_at: OffsetDateTime::now_utc().to_string(),
        media_items: request.media_items,
    }))
}

#[tracing::instrument(skip(session, state))]
pub async fn list_access_codes(
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
) -> Result<Json<AccessCodeListResponse>, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!(
            event = "access_denied",
            resource = "access_codes",
            action = "list",
            reason = "unauthenticated",
            "Unauthenticated attempt to list access codes"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    // Get access codes created by this user
    let codes = sqlx::query_as::<_, (i32, String, Option<String>, Option<String>, String)>(
        "SELECT id, code, description, expires_at, created_at FROM access_codes WHERE created_by = ? ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut access_codes = Vec::new();

    for (id, code, description, expires_at, created_at) in codes {
        // Get permissions for this code
        let permissions = sqlx::query_as::<_, (String, String)>(
            "SELECT media_type, media_slug FROM access_code_permissions WHERE access_code_id = ?",
        )
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let media_items = permissions
            .into_iter()
            .map(|(media_type, media_slug)| MediaItem {
                media_type,
                media_slug,
            })
            .collect();

        access_codes.push(AccessCodeResponse {
            id,
            code,
            description,
            expires_at,
            created_at,
            media_items,
        });
    }

    info!(count = access_codes.len(), user_id = %user_id, "Access codes listed");

    Ok(Json(AccessCodeListResponse { access_codes }))
}

#[tracing::instrument(skip(session, state))]
pub async fn delete_access_code(
    Path(code): Path<String>,
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
) -> Result<StatusCode, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        warn!(
            event = "access_denied",
            resource = "access_codes",
            action = "delete",
            code = %code,
            reason = "unauthenticated",
            "Unauthenticated attempt to delete access code"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user_id from session
    let user_id: String = session
        .get("user_id")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());

    // Delete access code (only if owned by current user)
    let rows_affected = sqlx::query("DELETE FROM access_codes WHERE code = ? AND created_by = ?")
        .bind(&code)
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .rows_affected();

    if rows_affected == 0 {
        warn!(
            event = "access_denied",
            resource = "access_codes",
            action = "delete",
            code = %code,
            user_id = %user_id,
            reason = "not_found_or_not_owner",
            "Access code not found or user doesn't own it"
        );
        Err(StatusCode::NOT_FOUND)
    } else {
        info!(
            event = "access_code_deleted",
            code = %code,
            user_id = %user_id,
            "Access code deleted successfully"
        );
        Ok(StatusCode::NO_CONTENT)
    }
}

pub fn access_code_routes(state: Arc<AccessCodeState>) -> Router {
    Router::new()
        .route("/api/access-codes", post(create_access_code))
        .route("/api/access-codes", get(list_access_codes))
        .route("/api/access-codes/:code", delete(delete_access_code))
        .with_state(state)
}

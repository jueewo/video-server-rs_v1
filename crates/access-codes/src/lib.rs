use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{self, info, warn};

// Import access control functionality
use access_control::{AccessContext, AccessControlService, Permission};
use common::ResourceType;

#[derive(Clone)]
pub struct AccessCodeState {
    pub pool: SqlitePool,
    pub access_control: Arc<AccessControlService>,
}

impl AccessCodeState {
    pub fn new(pool: SqlitePool) -> Self {
        let access_control = Arc::new(AccessControlService::with_audit_enabled(pool.clone(), true));
        Self {
            pool,
            access_control,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateAccessCodeRequest {
    pub code: String,
    pub description: Option<String>,
    pub expires_at: Option<String>, // ISO 8601 datetime string
    pub media_items: Vec<MediaItem>,
}

#[derive(Deserialize, Serialize, Clone)]
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

// Template structs
#[derive(Template)]
#[template(path = "codes/list.html")]
pub struct AccessCodesListTemplate {
    pub access_codes: Vec<AccessCodeDisplay>,
    pub total_pages: usize,
    pub current_page: usize,
    pub base_url: String,
}

#[derive(Clone)]
pub struct AccessCodeDisplay {
    pub code: String,
    pub description: String,
    pub has_description: bool,
    pub created_at: String,
    pub created_at_human: String,
    pub created_at_formatted: String,
    pub expires_at: String,
    pub expires_at_human: String,
    pub expires_at_formatted: String,
    pub has_expiration: bool,
    pub is_expired: bool,
    pub status: String,
    pub is_group_code: bool,
    pub group_name: String,
    pub resource_count: usize,
    pub usage_count: usize,
    pub media_items: Vec<MediaItem>,
}

// UI Page Handlers
#[tracing::instrument(skip(session, _state))]
pub async fn new_access_code_page(
    session: Session,
    State(_state): State<Arc<AccessCodeState>>,
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

    #[derive(Template)]
    #[template(path = "codes/new.html")]
    struct NewAccessCodeTemplate {}

    let template = NewAccessCodeTemplate {};
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

#[tracing::instrument(skip(session, state))]
pub async fn view_access_code_page(
    Path(code): Path<String>,
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
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
        .unwrap_or_else(|| "unknown".to_string());

    // Get access code details
    let code_record = sqlx::query_as::<_, (i32, String, Option<String>, Option<String>, String)>(
        "SELECT id, code, description, expires_at, created_at FROM access_codes WHERE code = ? AND created_by = ?"
    )
    .bind(&code)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (id, code_name, description, expires_at, created_at) =
        code_record.ok_or(StatusCode::NOT_FOUND)?;

    // Get permissions for this code
    let permissions = sqlx::query_as::<_, (String, String)>(
        "SELECT media_type, media_slug FROM access_code_permissions WHERE access_code_id = ?",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let media_items: Vec<MediaItem> = permissions
        .into_iter()
        .map(|(media_type, media_slug)| MediaItem {
            media_type,
            media_slug,
        })
        .collect();

    // Check if expired
    let is_expired = if let Some(ref exp) = expires_at {
        if let Ok(exp_dt) =
            OffsetDateTime::parse(exp, &time::format_description::well_known::Iso8601::DEFAULT)
        {
            exp_dt < OffsetDateTime::now_utc()
        } else {
            false
        }
    } else {
        false
    };

    // Format dates
    let created_at_human = format_human_date(&created_at);
    let created_at_formatted = format_full_date(&created_at);
    let expires_at_human = expires_at.as_ref().map(|exp| format_human_date(exp));
    let expires_at_formatted = expires_at.as_ref().map(|exp| format_full_date(exp));

    let code_display = AccessCodeDisplay {
        code: code_name.clone(),
        description: description.clone().unwrap_or_default(),
        has_description: description.is_some(),
        created_at: created_at.clone(),
        created_at_human,
        created_at_formatted,
        expires_at: expires_at.clone().unwrap_or_default(),
        expires_at_human: expires_at_human.clone().unwrap_or_default(),
        expires_at_formatted: expires_at_formatted.clone().unwrap_or_default(),
        has_expiration: expires_at.is_some(),
        is_expired,
        status: if is_expired {
            "expired".to_string()
        } else {
            "active".to_string()
        },
        is_group_code: false,
        group_name: String::new(),
        resource_count: media_items.len(),
        usage_count: 0,
        media_items,
    };

    #[derive(Template)]
    #[template(path = "codes/detail.html")]
    struct AccessCodeDetailTemplate {
        code: AccessCodeDisplay,
        base_url: String,
    }

    let template = AccessCodeDetailTemplate {
        code: code_display,
        base_url: "http://localhost:3000".to_string(), // TODO: Get from config
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
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

        // Validate ownership using AccessControlService
        // First get the resource ID
        let resource_id: Option<i32> = match item.media_type.as_str() {
            "video" => sqlx::query_scalar("SELECT id FROM videos WHERE slug = ?")
                .bind(&item.media_slug)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            "image" => sqlx::query_scalar("SELECT id FROM images WHERE slug = ?")
                .bind(&item.media_slug)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            _ => None,
        };

        let resource_id = resource_id.ok_or(StatusCode::NOT_FOUND)?;

        // Use AccessControlService to check Admin permission (ownership)
        let resource_type = match item.media_type.as_str() {
            "video" => ResourceType::Video,
            "image" => ResourceType::Image,
            _ => return Err(StatusCode::BAD_REQUEST),
        };

        let context = AccessContext::new(resource_type, resource_id).with_user(user_id.clone());

        let decision = state
            .access_control
            .check_access(context, Permission::Admin)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !decision.granted {
            warn!(
                event = "access_denied",
                resource = "media",
                action = "share",
                user_id = %user_id,
                media_type = %item.media_type,
                media_slug = %item.media_slug,
                reason = %decision.reason,
                "User attempted to share media they don't own"
            );
            return Err(StatusCode::FORBIDDEN);
        }

        info!(
            event = "ownership_validated",
            user_id = %user_id,
            media_type = %item.media_type,
            media_slug = %item.media_slug,
            "Ownership validated for access code creation"
        );

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

// UI Page Handlers
#[tracing::instrument(skip(session, state))]
pub async fn list_access_codes_page(
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
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

        let media_items: Vec<MediaItem> = permissions
            .into_iter()
            .map(|(media_type, media_slug)| MediaItem {
                media_type,
                media_slug,
            })
            .collect();

        // Check if expired
        let is_expired = if let Some(ref exp) = expires_at {
            if let Ok(exp_dt) =
                OffsetDateTime::parse(exp, &time::format_description::well_known::Iso8601::DEFAULT)
            {
                exp_dt < OffsetDateTime::now_utc()
            } else {
                false
            }
        } else {
            false
        };

        // Format human-readable dates
        let created_at_human = format_human_date(&created_at);
        let expires_at_human = expires_at.as_ref().map(|exp| format_human_date(exp));

        let created_at_formatted = format_full_date(&created_at);
        let expires_at_formatted = expires_at.as_ref().map(|exp| format_full_date(exp));

        access_codes.push(AccessCodeDisplay {
            code: code.clone(),
            description: description.clone().unwrap_or_default(),
            has_description: description.is_some(),
            created_at: created_at.clone(),
            created_at_human,
            created_at_formatted,
            expires_at: expires_at.clone().unwrap_or_default(),
            expires_at_human: expires_at_human.clone().unwrap_or_default(),
            expires_at_formatted: expires_at_formatted.unwrap_or_default(),
            has_expiration: expires_at.is_some(),
            is_expired,
            status: if is_expired {
                "expired".to_string()
            } else {
                "active".to_string()
            },
            is_group_code: false, // For now, all are individual codes
            group_name: String::new(),
            resource_count: media_items.len(),
            usage_count: 0, // TODO: Track usage in database
            media_items,
        });
    }

    let template = AccessCodesListTemplate {
        access_codes,
        total_pages: 1,
        current_page: 1,
        base_url: "http://localhost:3000".to_string(), // TODO: Get from config
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

fn format_human_date(date_str: &str) -> String {
    // Simple date formatting - can be improved
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

fn format_full_date(date_str: &str) -> String {
    // Format as "Jan 15, 2024 at 14:30"
    if let Ok(dt) = OffsetDateTime::parse(
        date_str,
        &time::format_description::well_known::Iso8601::DEFAULT,
    ) {
        // Simple formatting - could use time::format_description for more control
        format!(
            "{} {}, {} at {:02}:{:02}",
            match dt.month() {
                time::Month::January => "Jan",
                time::Month::February => "Feb",
                time::Month::March => "Mar",
                time::Month::April => "Apr",
                time::Month::May => "May",
                time::Month::June => "Jun",
                time::Month::July => "Jul",
                time::Month::August => "Aug",
                time::Month::September => "Sep",
                time::Month::October => "Oct",
                time::Month::November => "Nov",
                time::Month::December => "Dec",
            },
            dt.day(),
            dt.year(),
            dt.hour(),
            dt.minute()
        )
    } else {
        date_str.to_string()
    }
}

pub fn access_code_routes(state: Arc<AccessCodeState>) -> Router {
    Router::new()
        // API routes
        .route("/api/access-codes", post(create_access_code))
        .route("/api/access-codes", get(list_access_codes))
        .route("/api/access-codes/:code", delete(delete_access_code))
        // UI routes
        .route("/access/codes", get(list_access_codes_page))
        .route("/access/codes/new", get(new_access_code_page))
        .route("/access/codes/:code", get(view_access_code_page))
        .with_state(state)
}

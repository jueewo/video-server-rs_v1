use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Json},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{self, info, warn};

// Import access control functionality
use access_control::{AccessContext, AccessControlService, Permission};
use common::ResourceType;
pub use db::access_codes::{AccessCode, AccessCodePermission, AccessCodeRepository};
use db::media::MediaRepository;

#[derive(Clone)]
pub struct AccessCodeState {
    pub repo: Arc<dyn AccessCodeRepository>,
    pub media_repo: Arc<dyn MediaRepository>,
    pub access_control: Arc<AccessControlService>,
}

impl AccessCodeState {
    pub fn new(
        repo: Arc<dyn AccessCodeRepository>,
        media_repo: Arc<dyn MediaRepository>,
        access_control: Arc<AccessControlService>,
    ) -> Self {
        Self {
            repo,
            media_repo,
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
    /// When set, creates a folder-scoped code granting access to all media in the vault.
    /// media_items is ignored when vault_id is provided.
    pub vault_id: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MediaItem {
    pub media_type: String, // "video", "image", or "document"
    pub media_slug: String,
    pub filename: String, // For file type detection (PDF, MD, BPMN, etc.)
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub title: String,
}

impl MediaItem {
    /// Get the specific file type label for display (PDF, MD, BPMN, etc.)
    pub fn file_type_label(&self) -> &str {
        if self.media_type == "video" {
            "VIDEO"
        } else if self.media_type == "image" {
            "IMAGE"
        } else if self.media_type == "document" {
            // Check file extension for specific document types
            if self.filename.ends_with(".pdf") {
                "PDF"
            } else if self.filename.ends_with(".md") || self.filename.ends_with(".markdown") {
                "MD"
            } else if self.filename.ends_with(".bpmn") {
                "BPMN"
            } else {
                "DOC"
            }
        } else {
            "FILE"
        }
    }

    /// Get badge color class for the file type
    pub fn badge_color(&self) -> &str {
        match self.file_type_label() {
            "VIDEO" => "badge-error",
            "IMAGE" => "badge-success",
            "PDF" => "badge-warning",
            "MD" => "badge-secondary",
            "BPMN" => "badge-info",
            _ => "badge-accent",
        }
    }
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

#[derive(Clone)]
pub struct ResourcePreview {
    pub media_type: String,
    pub slug: String,
    pub title: String,
}

// Template structs
#[derive(Template, Clone)]
#[template(path = "codes/preview.html")]
pub struct PreviewTemplate {
    pub authenticated: bool,
    pub code: String,
    pub description: String,
    pub has_description: bool,
    pub resource_count: usize,
    pub resources: Vec<ResourcePreview>,
    pub base_url: String,
}

#[derive(Template)]
#[template(path = "codes/list.html")]
pub struct AccessCodesListTemplate {
    pub authenticated: bool,
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
    struct NewAccessCodeTemplate {
        authenticated: bool,
    }

    let template = NewAccessCodeTemplate {
        authenticated: true,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

#[tracing::instrument(skip(session, state, headers))]
pub async fn view_access_code_page(
    Path(code): Path<String>,
    headers: HeaderMap,
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
    let ac = state
        .repo
        .get_code_by_code_and_user(&code, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let id = ac.id;
    let code_name = ac.code.clone();
    let description = ac.description.clone();
    let expires_at = ac.expires_at.clone();
    let created_at = ac.created_at.clone();

    // Get permissions then enrich with media_items data for display
    let media_items = enrich_permissions(&state, id).await?;

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
        authenticated: bool,
        code: AccessCodeDisplay,
        base_url: String,
    }

    let template = AccessCodeDetailTemplate {
        authenticated: true, // Detail page requires auth
        code: code_display,
        base_url: get_base_url(&headers),
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Public preview page for access code - shows all resources available with this code
/// This is the page users land on when they click the shared access code URL
pub async fn preview_access_code_page(
    Query(params): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
    State(state): State<Arc<AccessCodeState>>,
) -> Result<Html<String>, StatusCode> {
    let code = params
        .get("code")
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();

    // Get access code details (no auth required - this is public)
    let ac = state
        .repo
        .get_active_code(&code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let id = ac.id;
    let code_name = ac.code.clone();
    let description = ac.description.clone();
    let expires_at = ac.expires_at.clone();

    // Check if expired
    if let Some(ref exp) = expires_at {
        if let Ok(exp_dt) =
            OffsetDateTime::parse(exp, &time::format_description::well_known::Iso8601::DEFAULT)
        {
            if exp_dt < OffsetDateTime::now_utc() {
                return Err(StatusCode::GONE); // 410 Gone for expired codes
            }
            false
        } else {
            false
        }
    } else {
        false
    };

    // Get permissions then enrich with media_items data for display
    let perms = state
        .repo
        .get_permissions(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut resources = Vec::new();
    for p in perms {
        let title: Option<String> = state
            .media_repo
            .get_media_title(&p.media_slug, &p.media_type)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        resources.push(ResourcePreview {
            media_type: p.media_type,
            slug: p.media_slug.clone(),
            title: title.unwrap_or_else(|| p.media_slug),
        });
    }

    let template = PreviewTemplate {
        authenticated: false, // Preview page is public
        code: code_name,
        description: description.clone().unwrap_or_default(),
        has_description: description.is_some(),
        resource_count: resources.len(),
        resources,
        base_url: get_base_url(&headers),
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
    let exists = state
        .repo
        .code_exists(&request.code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if exists {
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
    let expires_at_str = expires_at.map(|dt| {
        dt.format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap()
    });
    let code_id = state
        .repo
        .create_code(
            &request.code,
            request.description.as_deref(),
            expires_at_str.as_deref(),
            &user_id,
            request.vault_id.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Folder-scoped code: no per-item permissions needed
    if request.vault_id.is_some() {
        info!(
            event = "access_code_created",
            code = %request.code,
            user_id = %user_id,
            scope = "folder",
            vault_id = ?request.vault_id,
            "Folder-scoped access code created"
        );
        return Ok(Json(AccessCodeResponse {
            id: code_id,
            code: request.code,
            description: request.description,
            expires_at: request.expires_at,
            created_at: OffsetDateTime::now_utc().to_string(),
            media_items: vec![],
        }));
    }

    // Insert permissions (only for owned media)
    for item in &request.media_items {
        if !["video", "image", "document"].contains(&item.media_type.as_str()) {
            warn!(
                event = "invalid_request",
                media_type = %item.media_type,
                "Invalid media type in access code request"
            );
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate ownership using AccessControlService
        // First get the resource ID
        let resource_id: i64 = state
            .media_repo
            .get_media_id_by_type(&item.media_type, &item.media_slug)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Use AccessControlService to check Admin permission (ownership)
        let resource_type = match item.media_type.as_str() {
            "video" => ResourceType::Video,
            "image" => ResourceType::Image,
            "document" => ResourceType::File,
            _ => return Err(StatusCode::BAD_REQUEST),
        };

        let context = AccessContext::new(resource_type, resource_id as i32).with_user(user_id.clone());

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

        // Insert into access_code_permissions (used by access control system)
        state
            .repo
            .add_permission(code_id, &item.media_type, &item.media_slug)
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
    let codes = state
        .repo
        .list_user_codes(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut access_codes = Vec::new();

    for ac in codes {
        // Get permissions then enrich with media_items data for display
        let media_items = enrich_permissions(&state, ac.id).await?;

        access_codes.push(AccessCodeResponse {
            id: ac.id,
            code: ac.code,
            description: ac.description,
            expires_at: ac.expires_at,
            created_at: ac.created_at,
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
    let deleted = state
        .repo
        .delete_code(&code, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !deleted {
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
#[tracing::instrument(skip(session, state, headers))]
pub async fn list_access_codes_page(
    session: Session,
    headers: HeaderMap,
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
    let codes = state
        .repo
        .list_user_codes(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut access_codes = Vec::new();

    for ac in codes {
        // Get permissions then enrich with media_items data for display
        let media_items = enrich_permissions(&state, ac.id).await?;

        let description = ac.description;
        let expires_at = ac.expires_at;
        let created_at = ac.created_at;
        let current_downloads = ac.current_downloads;

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
            code: ac.code.clone(),
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
            usage_count: current_downloads as usize,
            media_items,
        });
    }

    let template = AccessCodesListTemplate {
        authenticated: true, // List page requires auth
        access_codes,
        total_pages: 1,
        current_page: 1,
        base_url: get_base_url(&headers),
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Fetch permissions from the repo and enrich each with filename/thumbnail/title
/// from the media_items table via `MediaRepository`.
async fn enrich_permissions(
    state: &AccessCodeState,
    code_id: i32,
) -> Result<Vec<MediaItem>, StatusCode> {
    let perms = state
        .repo
        .get_permissions(code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut items = Vec::with_capacity(perms.len());
    for p in perms {
        let enrichment = state
            .media_repo
            .get_media_enrichment(&p.media_slug)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let (filename, thumbnail_url, title) = match enrichment {
            Some(e) => (e.filename, e.thumbnail_url, e.title),
            None => (p.media_slug.clone(), None, p.media_slug.clone()),
        };

        items.push(MediaItem {
            media_type: p.media_type,
            media_slug: p.media_slug,
            filename: filename.to_lowercase(),
            thumbnail_url,
            title,
        });
    }
    Ok(items)
}

/// Extract base URL from request headers
/// Falls back to localhost:3000 for development if headers are missing
fn get_base_url(headers: &HeaderMap) -> String {
    // Try to get host from headers
    if let Some(host) = headers.get("host").and_then(|h| h.to_str().ok()) {
        // Check if request was made over HTTPS (via X-Forwarded-Proto header from reverse proxy)
        let scheme = headers
            .get("x-forwarded-proto")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http");

        return format!("{}://{}", scheme, host);
    }

    // Fallback for development
    "http://localhost:3000".to_string()
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

#[tracing::instrument(skip(session, state, item))]
pub async fn add_media_to_code(
    Path(code): Path<String>,
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
    Json(item): Json<MediaItem>,
) -> Result<StatusCode, StatusCode> {
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
        .unwrap_or_else(|| "unknown".to_string());

    if !["video", "image", "document"].contains(&item.media_type.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get access code id (must belong to user)
    let code_id = state
        .repo
        .get_code_id_for_user(&code, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify media exists and user owns it
    let resource_id: i64 = state
        .media_repo
        .get_media_id_by_type(&item.media_type, &item.media_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let resource_type = match item.media_type.as_str() {
        "video" => ResourceType::Video,
        "image" => ResourceType::Image,
        "document" => ResourceType::File,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let context = AccessContext::new(resource_type, resource_id as i32).with_user(user_id.clone());

    let decision = state
        .access_control
        .check_access(context, Permission::Admin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !decision.granted {
        warn!(
            event = "access_denied",
            user_id = %user_id,
            media_slug = %item.media_slug,
            "User attempted to add media they don't own to access code"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .repo
        .add_permission(code_id, &item.media_type, &item.media_slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(
        event = "media_added_to_code",
        code = %code,
        user_id = %user_id,
        media_slug = %item.media_slug,
        "Media added to access code"
    );

    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(session, state))]
pub async fn remove_media_from_code(
    Path((code, slug)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<AccessCodeState>>,
) -> Result<StatusCode, StatusCode> {
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
        .unwrap_or_else(|| "unknown".to_string());

    // Get access code id (must belong to user)
    let code_id = state
        .repo
        .get_code_id_for_user(&code, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let removed = state
        .repo
        .remove_permission(code_id, &slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !removed {
        return Err(StatusCode::NOT_FOUND);
    }

    info!(
        event = "media_removed_from_code",
        code = %code,
        user_id = %user_id,
        media_slug = %slug,
        "Media removed from access code"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Protected access-code routes (require authentication)
pub fn access_code_routes(state: Arc<AccessCodeState>) -> Router {
    Router::new()
        // API routes
        .route("/api/access-codes", post(create_access_code))
        .route("/api/access-codes", get(list_access_codes))
        .route("/api/access-codes/{code}", delete(delete_access_code))
        .route("/api/access-codes/{code}/media", post(add_media_to_code))
        .route(
            "/api/access-codes/{code}/media/{slug}",
            delete(remove_media_from_code),
        )
        // UI routes
        .route("/access/codes", get(list_access_codes_page))
        .route("/access/codes/new", get(new_access_code_page))
        .route("/access/codes/{code}", get(view_access_code_page))
        .with_state(state)
}

/// Public access-code routes (no authentication required)
/// The preview page is intentionally public — it's the landing page for
/// shared access-code links.
pub fn access_code_public_routes(state: Arc<AccessCodeState>) -> Router {
    Router::new()
        .route("/access/preview", get(preview_access_code_page))
        .with_state(state)
}

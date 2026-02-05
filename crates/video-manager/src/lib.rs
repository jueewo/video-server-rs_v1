use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::{path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tokio_util::io::ReaderStream;
use tower_sessions::Session;
use tracing::{self, info};

// Import access control functionality
use access_control::{AccessContext, AccessControlService, Permission};
use common::ResourceType;

// Import tag functionality from common crate
use common::{
    models::tag::{AddTagsRequest, Tag},
    services::tag_service::TagService,
};

// -------------------------------
// Template Structs
// -------------------------------

#[derive(Template)]
#[template(path = "videos/list-tailwind.html")]
pub struct VideoListTemplate {
    authenticated: bool,
    page_title: String,
    public_videos: Vec<(String, String, i32)>,
    private_videos: Vec<(String, String, i32)>,
}

#[derive(Template)]
#[template(path = "videos/player.html")]
pub struct VideoPlayerTemplate {
    authenticated: bool,
    title: String,
    slug: String,
    is_public: bool,
}

#[derive(Template)]
#[template(path = "videos/live_test.html")]
pub struct LiveTestTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "unauthorized.html")]
pub struct UnauthorizedTemplate {
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct NotFoundTemplate {
    authenticated: bool,
}

// -------------------------------
// Configuration Constants
// -------------------------------
pub const RTMP_PUBLISH_TOKEN: &str = "supersecret123"; // Change this to a strong secret!
pub const LIVE_STREAM_KEY: &str = "live"; // URL slug for live: /hls/live/index.m3u8
pub const MEDIAMTX_HLS_URL: &str = "http://localhost:8888"; // MediaMTX HLS endpoint
pub const MEDIAMTX_API_URL: &str = "http://localhost:9997"; // MediaMTX API endpoint

// -------------------------------
// Shared State
// -------------------------------
#[derive(Clone)]
pub struct VideoManagerState {
    pub pool: Pool<Sqlite>,
    pub storage_dir: PathBuf,
    pub http_client: Client,
    pub access_control: Arc<AccessControlService>,
}

impl VideoManagerState {
    pub fn new(pool: Pool<Sqlite>, storage_dir: PathBuf, http_client: Client) -> Self {
        let access_control = Arc::new(AccessControlService::with_audit_enabled(pool.clone(), true));
        Self {
            pool,
            storage_dir,
            http_client,
            access_control,
        }
    }
}

// -------------------------------
// Router Setup
// -------------------------------
pub fn video_routes() -> Router<Arc<VideoManagerState>> {
    Router::new()
        .route("/videos", get(videos_list_handler))
        .route("/watch/:slug", get(video_player_handler))
        .route("/test", get(live_test_handler))
        .route("/hls/*path", get(hls_proxy_handler))
        .route("/api/stream/validate", get(validate_stream_handler))
        .route("/api/stream/authorize", get(authorize_stream_handler))
        .route("/api/mediamtx/status", get(mediamtx_status))
        // Video tag endpoints
        .route("/api/videos/:id/tags", get(get_video_tags_handler))
        .route("/api/videos/:id/tags", post(add_video_tags_handler))
        .route("/api/videos/:id/tags", put(replace_video_tags_handler))
        .route(
            "/api/videos/:id/tags/:tag_slug",
            delete(remove_video_tag_handler),
        )
}

// -------------------------------
// MediaMTX Authentication Endpoints
// -------------------------------

// Validate stream publisher (called by MediaMTX via runOnInit)
#[tracing::instrument(skip(params))]
async fn validate_stream_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;

    if token == RTMP_PUBLISH_TOKEN {
        println!("✅ Stream publisher authorized: token={}", token);
        Ok(StatusCode::OK)
    } else {
        println!("❌ Stream publisher rejected: invalid token");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Authorize stream viewer (called by MediaMTX via runOnRead)
#[tracing::instrument(skip(session))]
async fn authorize_stream_handler(session: Session) -> Result<StatusCode, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if authenticated {
        let user_id: Option<u32> = session.get("user_id").await.ok().flatten();
        println!("✅ Stream viewer authorized: user_id={:?}", user_id);
        Ok(StatusCode::OK)
    } else {
        println!("❌ Stream viewer rejected: not authenticated");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// -------------------------------
// Video Listing Page Handler
// -------------------------------

#[tracing::instrument(skip(session, state))]
pub async fn videos_list_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<VideoListTemplate, StatusCode> {
    // Check if user is authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get user_id from session
    let user_id: Option<String> = if authenticated {
        session.get("user_id").await.ok().flatten()
    } else {
        None
    };

    // Get videos based on authentication and ownership
    let videos = get_videos(&state.pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(
        count = videos.len(),
        authenticated = authenticated,
        "Videos loaded"
    );

    let page_title = if authenticated {
        "🎥 All Videos".to_string()
    } else {
        "🎥 Public Videos".to_string()
    };

    // Separate videos into public and private
    let mut public_videos: Vec<(String, String, i32)> = Vec::new();
    let mut private_videos: Vec<(String, String, i32)> = Vec::new();

    for video in videos {
        if video.2 == 1 {
            public_videos.push(video);
        } else {
            private_videos.push(video);
        }
    }

    Ok(VideoListTemplate {
        authenticated,
        page_title,
        public_videos,
        private_videos,
    })
}

#[derive(Deserialize)]
pub struct AccessCodeQuery {
    access_code: Option<String>,
}

// -------------------------------
// Video Player Page Handler
// -------------------------------

#[tracing::instrument(skip(query, session, state))]
pub async fn video_player_handler(
    Path(slug): Path<String>,
    Query(query): Query<AccessCodeQuery>,
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<VideoPlayerTemplate, Response> {
    // Check authentication first
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get user_id from session if authenticated
    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    // Lookup video in database - get id, title, and is_public
    let video: Option<(i32, String, i32)> =
        sqlx::query_as("SELECT id, title, is_public FROM videos WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;

    let (video_id, title, is_public_int) = video.ok_or_else(|| {
        (StatusCode::NOT_FOUND, NotFoundTemplate { authenticated }).into_response()
    })?;
    let is_public = is_public_int == 1;

    // Build access context for modern access control
    let mut context = AccessContext::new(ResourceType::Video, video_id);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }
    if let Some(key) = query.access_code.clone() {
        context = context.with_key(key);
    }

    // Check access using the 4-layer access control system
    let decision = state
        .access_control
        .check_access(context, Permission::Read)
        .await
        .map_err(|e| {
            info!(error = ?e, "Access control error");
            (StatusCode::INTERNAL_SERVER_ERROR, "Access check failed").into_response()
        })?;

    if !decision.granted {
        info!(
            video_slug = %slug,
            reason = %decision.reason,
            layer_checked = ?decision.layer,
            "Access denied to video"
        );
        return Err((
            StatusCode::UNAUTHORIZED,
            UnauthorizedTemplate {
                authenticated: false,
            },
        )
            .into_response());
    }

    // Log successful access with layer information
    info!(
        video_slug = %slug,
        access_layer = ?decision.layer,
        reason = %decision.reason,
        "Access granted to video"
    );

    Ok(VideoPlayerTemplate {
        authenticated,
        title,
        slug,
        is_public,
    })
}

// -------------------------------
// Live Stream Test Handler
// -------------------------------

#[tracing::instrument(skip(session))]
pub async fn live_test_handler(session: Session) -> Result<LiveTestTemplate, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    Ok(LiveTestTemplate { authenticated })
}

// -------------------------------
// HLS Proxy Handler for Live Streams and VOD
// -------------------------------

#[tracing::instrument(skip(query, session, state))]
pub async fn hls_proxy_handler(
    Path(path): Path<String>,
    Query(query): Query<AccessCodeQuery>,
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<Response, StatusCode> {
    // Parse the path to extract slug and file
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let slug = parts[0];
    let file_path = parts[1];

    // Handle live stream - proxy to MediaMTX
    if slug == LIVE_STREAM_KEY {
        // Check authentication for live stream
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            println!("❌ HLS request rejected: not authenticated");
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Proxy request to MediaMTX
        let mediamtx_url = format!("{}/{}/{}", MEDIAMTX_HLS_URL, slug, file_path);

        println!("📡 Proxying HLS request: {}", mediamtx_url);

        let response = state
            .http_client
            .get(&mediamtx_url)
            .send()
            .await
            .map_err(|e| {
                println!("❌ MediaMTX proxy error: {}", e);
                StatusCode::BAD_GATEWAY
            })?;

        // Check if MediaMTX returned an error
        if !response.status().is_success() {
            println!("❌ MediaMTX returned error: {}", response.status());
            return Err(
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
            );
        }

        // Determine content type
        let content_type = if file_path.ends_with(".m3u8") {
            "application/vnd.apple.mpegurl"
        } else if file_path.ends_with(".ts") {
            "video/MP2T"
        } else {
            "application/octet-stream"
        };

        // Get response body
        let bytes = response
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        // Build response with proper headers
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(
                header::CACHE_CONTROL,
                if file_path.ends_with(".m3u8") {
                    "no-cache, no-store, must-revalidate"
                } else {
                    "max-age=10"
                },
            )
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
            .body(axum::body::Body::from(bytes))
            .unwrap());
    }

    // Handle VOD - serve from local storage
    // DB lookup for regular videos - get id and is_public
    let video: Option<(i32, i32)> =
        sqlx::query_as("SELECT id, is_public FROM videos WHERE slug = ?")
            .bind(slug)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (video_id, is_public_int) = video.ok_or(StatusCode::NOT_FOUND)?;
    let is_public = is_public_int == 1;

    // Get user_id from session if authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let user_id: Option<String> = if authenticated {
        session.get::<String>("user_id").await.ok().flatten()
    } else {
        None
    };

    // Build access context for modern access control
    // For HLS streaming, we require Download permission
    let mut context = AccessContext::new(ResourceType::Video, video_id);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }
    if let Some(key) = query.access_code.clone() {
        context = context.with_key(key);
    }

    // Check access using the 4-layer access control system
    let decision = state
        .access_control
        .check_access(context, Permission::Download)
        .await
        .map_err(|e| {
            info!(error = ?e, "Access control error for HLS stream");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !decision.granted {
        info!(
            video_slug = %slug,
            file_path = %file_path,
            reason = %decision.reason,
            "Access denied to HLS stream"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!(
        video_slug = %slug,
        access_layer = ?decision.layer,
        "Access granted to HLS stream"
    );

    // Serve VOD file from storage (single folder structure)
    let full_path = state.storage_dir.join("videos").join(slug).join(file_path);

    // Check if file exists and read it
    let file = tokio::fs::File::open(&full_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Determine content type
    let content_type = if file_path.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if file_path.ends_with(".ts") {
        "video/MP2T"
    } else {
        "application/octet-stream"
    };

    // Stream the file
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=3600")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .body(body)
        .unwrap())
}

// -------------------------------
// MediaMTX Status Endpoint
// -------------------------------

#[tracing::instrument(skip(state))]
pub async fn mediamtx_status(
    State(state): State<Arc<VideoManagerState>>,
) -> Result<String, StatusCode> {
    let url = format!("{}/v3/paths/list", MEDIAMTX_API_URL);

    let response = state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let text = response.text().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(text)
}

// -------------------------------
// Helper Functions
// -------------------------------

pub async fn check_access_code(
    pool: &Pool<Sqlite>,
    code: &str,
    media_type: &str,
    media_slug: &str,
) -> bool {
    // Check if access code exists and hasn't expired
    let access_code: Option<(i32, Option<String>)> =
        sqlx::query_as("SELECT id, expires_at FROM access_codes WHERE code = ?")
            .bind(code)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    if let Some((code_id, expires_at)) = access_code {
        // Check expiration
        if let Some(expiry_str) = &expires_at {
            match OffsetDateTime::parse(
                &expiry_str,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                Ok(expiry) => {
                    let now = OffsetDateTime::now_utc();
                    if expiry < now {
                        return false; // Code has expired
                    }
                }
                Err(_) => {
                    return false; // Invalid expiry format
                }
            }
        }

        // Check if this code grants access to the specific media item
        let permission: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM access_code_permissions
             WHERE access_code_id = ? AND media_type = ? AND media_slug = ?",
        )
        .bind(code_id)
        .bind(media_type)
        .bind(media_slug)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        let has_access = permission.is_some();
        if has_access {
            info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
        }
        has_access
    } else {
        false
    }
}

pub async fn get_videos(
    pool: &Pool<Sqlite>,
    user_id: Option<String>,
) -> Result<Vec<(String, String, i32)>, sqlx::Error> {
    match user_id {
        Some(uid) => {
            // Show public videos + user's private videos
            sqlx::query_as(
                "SELECT slug, title, is_public FROM videos
                 WHERE is_public = 1 OR user_id = ?
                 ORDER BY is_public DESC, title",
            )
            .bind(uid)
            .fetch_all(pool)
            .await
        }
        None => {
            // Show only public videos for unauthenticated users
            sqlx::query_as(
                "SELECT slug, title, is_public FROM videos
                 WHERE is_public = 1
                 ORDER BY title",
            )
            .fetch_all(pool)
            .await
        }
    }
}

// -------------------------------
// Video Tag Handlers
// -------------------------------

#[derive(Debug, Serialize)]
struct VideoTagsResponse {
    video_id: i32,
    tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(sqlx::FromRow)]
struct VideoRecord {
    id: i32,
    user_id: Option<String>,
    is_public: i32,
}

/// Helper function to check if user can modify video tags
/// Uses modern AccessControlService with Edit permission
async fn can_modify_video(
    pool: &Pool<Sqlite>,
    video_id: i32,
    user_sub: &str,
) -> Result<bool, sqlx::Error> {
    // Use the new access control service
    let access_control = AccessControlService::new(pool.clone());

    let context = AccessContext::new(ResourceType::Video, video_id).with_user(user_sub.to_string());

    match access_control.check_access(context, Permission::Edit).await {
        Ok(decision) => Ok(decision.granted),
        Err(_) => Ok(false),
    }
}

/// Helper to get user from session
async fn get_user_from_session(session: &Session, pool: &Pool<Sqlite>) -> Option<String> {
    let user_sub: Option<String> = session.get("user_sub").await.ok().flatten();

    if let Some(sub) = user_sub {
        // Verify user exists
        let exists: Option<(String,)> = sqlx::query_as("SELECT sub FROM users WHERE sub = ?")
            .bind(&sub)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

        exists.map(|(s,)| s)
    } else {
        None
    }
}

/// GET /api/videos/:id/tags - Get all tags for a video
#[tracing::instrument(skip(state, _session))]
pub async fn get_video_tags_handler(
    State(state): State<Arc<VideoManagerState>>,
    _session: Session,
    Path(video_id): Path<i32>,
) -> Result<Json<VideoTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if video exists
    let video_exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM videos WHERE id = ?")
        .bind(video_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if video_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Video not found".to_string(),
            }),
        ));
    }

    // Get tags for this video
    let service = TagService::new(&state.pool);
    let tags = service
        .get_video_tags(video_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    Ok(Json(VideoTagsResponse { video_id, tags }))
}

/// POST /api/videos/:id/tags - Add tags to a video
#[tracing::instrument(skip(state, session))]
pub async fn add_video_tags_handler(
    State(state): State<Arc<VideoManagerState>>,
    session: Session,
    Path(video_id): Path<i32>,
    Json(request): Json<AddTagsRequest>,
) -> Result<Json<VideoTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool).await.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required".to_string(),
        }),
    ))?;

    // Check if user can modify this video
    let can_modify = can_modify_video(&state.pool, video_id, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to modify this video".to_string(),
            }),
        ));
    }

    // Add tags to video
    let service = TagService::new(&state.pool);
    service
        .add_tags_to_video(video_id, request.tag_names, Some(&user_sub))
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    // Get updated tag list
    let tags = service
        .get_video_tags(video_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    info!("Added {} tags to video {}", tags.len(), video_id);
    Ok(Json(VideoTagsResponse { video_id, tags }))
}

/// PUT /api/videos/:id/tags - Replace all tags on a video
#[tracing::instrument(skip(state, session))]
pub async fn replace_video_tags_handler(
    State(state): State<Arc<VideoManagerState>>,
    session: Session,
    Path(video_id): Path<i32>,
    Json(request): Json<AddTagsRequest>,
) -> Result<Json<VideoTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool).await.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required".to_string(),
        }),
    ))?;

    // Check if user can modify this video
    let can_modify = can_modify_video(&state.pool, video_id, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to modify this video".to_string(),
            }),
        ));
    }

    // Replace all tags
    let service = TagService::new(&state.pool);
    service
        .replace_video_tags(video_id, request.tag_names, Some(&user_sub))
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    // Get updated tag list
    let tags = service
        .get_video_tags(video_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    info!(
        "Replaced tags on video {} with {} tags",
        video_id,
        tags.len()
    );
    Ok(Json(VideoTagsResponse { video_id, tags }))
}

/// DELETE /api/videos/:id/tags/:tag_slug - Remove a tag from a video
#[tracing::instrument(skip(state, session))]
pub async fn remove_video_tag_handler(
    State(state): State<Arc<VideoManagerState>>,
    session: Session,
    Path((video_id, tag_slug)): Path<(i32, String)>,
) -> Result<Json<VideoTagsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get authenticated user
    let user_sub = get_user_from_session(&session, &state.pool).await.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Authentication required".to_string(),
        }),
    ))?;

    // Check if user can modify this video
    let can_modify = can_modify_video(&state.pool, video_id, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to modify this video".to_string(),
            }),
        ));
    }

    // Remove tag from video
    let service = TagService::new(&state.pool);
    let removed = service
        .remove_tag_from_video(video_id, &tag_slug)
        .await
        .map_err(|e: String| {
            let status = if e.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(ErrorResponse { error: e }))
        })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Tag '{}' not associated with video {}", tag_slug, video_id),
            }),
        ));
    }

    // Get updated tag list
    let tags = service
        .get_video_tags(video_id)
        .await
        .map_err(|e: String| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    info!("Removed tag '{}' from video {}", tag_slug, video_id);
    Ok(Json(VideoTagsResponse { video_id, tags }))
}

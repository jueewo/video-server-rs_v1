// Module declarations
pub mod cleanup;
pub mod errors;
pub mod ffmpeg;
pub mod hls;
pub mod media_item_impl;
pub mod metrics;
pub mod processing;
pub mod progress;
pub mod retry;
pub mod storage;
pub mod upload;
pub mod upload_v2;

// Re-export key processing types for external use
pub use processing::{ProcessingContext, ProcessingStage, process_video};

use askama::Template;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{Pool, Row, Sqlite};
use std::{path::PathBuf, sync::Arc};
use tokio_util::io::ReaderStream;
use tower_sessions::Session;
use tracing::{self, info};

// Import access control functionality
use access_control::{AccessContext, AccessControlService, Permission};
use common::ResourceType;
use db::media::MediaRepository;


// Import upload module types
use crate::progress::{ProgressTracker, UploadProgress};
use crate::upload::{handle_video_upload, UploadState};

// -------------------------------
// Template Structs
// -------------------------------

#[derive(Template)]
#[template(path = "videos/list-tailwind.html")]
pub struct VideoListTemplate {
    authenticated: bool,
    page_title: String,
    page_subtitle: String,
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
#[template(path = "videos/edit.html")]
pub struct VideoEditTemplate {
    #[allow(dead_code)]
    authenticated: bool,
    video: VideoDetail,
}

#[derive(Template)]
#[template(path = "videos/new.html")]
pub struct VideoNewTemplate {
    #[allow(dead_code)]
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "videos/upload-enhanced.html")]
pub struct VideoUploadTemplate {
    #[allow(dead_code)]
    authenticated: bool,
}

#[derive(Template)]
#[template(path = "videos/upload-simple.html")]
pub struct SimpleVideoUploadTemplate {
    #[allow(dead_code)]
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
// Video Detail Struct
// -------------------------------
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoDetail {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub is_public: bool,
    pub user_id: Option<String>,
    pub group_id: Option<i32>,
    pub group_id_str: String,
    pub duration: Option<i64>,
    pub file_size: Option<i64>,
    pub resolution: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<i32>,
    pub codec: Option<String>,
    pub thumbnail_url: Option<String>,
    pub poster_url: Option<String>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub status: String,
    pub featured: bool,
    pub allow_comments: bool,
    pub allow_download: bool,
    pub mature_content: bool,
    pub view_count: i64,
    pub like_count: i64,
    pub download_count: i64,
    pub share_count: i64,
    pub upload_date: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
}

impl VideoDetail {
    pub fn group_id_str(&self) -> String {
        self.group_id.map(|id| id.to_string()).unwrap_or_default()
    }
}

// -------------------------------
// Configuration Constants
// -------------------------------
/// RTMP publish token — loaded from `RTMP_PUBLISH_TOKEN` env var.
/// Falls back to `"supersecret123"` ONLY in development; production startup
/// must reject this default (enforced in `main.rs` startup validation).
pub fn rtmp_publish_token() -> String {
    std::env::var("RTMP_PUBLISH_TOKEN").unwrap_or_else(|_| "supersecret123".to_string())
}

pub const LIVE_STREAM_KEY: &str = "live"; // URL slug for live: /hls/live/index.m3u8
pub const MEDIAMTX_HLS_URL: &str = "http://localhost:8888"; // MediaMTX HLS endpoint
pub const MEDIAMTX_API_URL: &str = "http://localhost:9997"; // MediaMTX API endpoint

// -------------------------------
// Shared State
// -------------------------------
#[derive(Clone)]
pub struct VideoManagerState {
    pub pool: Pool<Sqlite>,
    pub repo: Arc<dyn MediaRepository>,
    pub vault_repo: Arc<dyn db::vaults::VaultRepository>,
    pub storage_dir: PathBuf,
    pub http_client: Client,
    pub access_control: Arc<AccessControlService>,
    pub progress_tracker: ProgressTracker,
    pub storage_config: storage::StorageConfig,
    pub ffmpeg_config: ffmpeg::FFmpegConfig,
    pub hls_config: hls::HlsConfig,
    pub metrics_store: metrics::MetricsStore,
    pub audit_logger: metrics::AuditLogger,
    /// RTMP stream publishing token, loaded from env at construction time.
    pub rtmp_publish_token: String,
}

impl VideoManagerState {
    pub fn new(pool: Pool<Sqlite>, repo: Arc<dyn MediaRepository>, vault_repo: Arc<dyn db::vaults::VaultRepository>, storage_dir: PathBuf, http_client: Client, access_control: Arc<AccessControlService>) -> Self {
        let progress_tracker = ProgressTracker::default();

        // Start automatic cleanup task (runs every 5 minutes)
        progress_tracker.start_cleanup_task(300);

        // Initialize storage configuration
        let storage_config = storage::StorageConfig::new(storage_dir.clone());

        // Initialize FFmpeg configuration
        let ffmpeg_config = ffmpeg::FFmpegConfig {
            ffmpeg_path: std::path::PathBuf::from("ffmpeg"),
            ffprobe_path: std::path::PathBuf::from("ffprobe"),
            threads: 0, // 0 = auto
        };

        // Initialize HLS configuration
        let hls_config = hls::HlsConfig {
            segment_duration: 6,
            auto_quality_selection: true,
            delete_original: false,
        };

        // Initialize metrics store
        let metrics_store = metrics::ProcessingMetrics::new_store();

        // Initialize audit logger
        let audit_logger = metrics::AuditLogger::new();

        Self {
            pool,
            repo,
            vault_repo,
            storage_dir,
            http_client,
            access_control,
            progress_tracker,
            storage_config,
            ffmpeg_config,
            hls_config,
            metrics_store,
            audit_logger,
            rtmp_publish_token: rtmp_publish_token(),
        }
    }
}

// -------------------------------
// Router Setup
// -------------------------------
pub fn video_routes() -> Router<Arc<VideoManagerState>> {
    Router::new()
        .route("/videos", get(videos_list_handler))
        .route("/videos/new", get(video_new_page_handler))
        // Video upload endpoints
        .route("/videos/upload", get(video_upload_page_handler))
        // Legacy video detail routes - REMOVED: Use unified /media/{slug} instead
        // .route("/videos/{slug}", ...) - now at /media/{slug}
        // .route("/videos/{slug}/edit", ...) - now at /media/{slug}/edit
        // .route("/watch/{slug}", ...) - now at /media/{slug}
        .route("/test", get(live_test_handler))
        .route("/hls/{*path}", get(hls_proxy_handler))
        .route("/api/stream/validate", get(validate_stream_handler))
        .route("/api/stream/authorize", get(authorize_stream_handler))
        .route("/api/mediamtx/status", get(mediamtx_status))
        // Video CRUD API
        .route("/api/videos", get(list_videos_api_handler))
        .route("/api/videos", post(register_video_handler))
        // Video upload endpoints
        .route("/api/videos/upload", post(upload::handle_video_upload_vms))
        .route(
            "/api/videos/upload/{upload_id}/progress",
            get(get_upload_progress_handler),
        )
        .route("/api/videos/metrics", get(get_metrics_handler))
        .route(
            "/api/videos/metrics/detailed",
            get(get_detailed_metrics_handler),
        )
        .route(
            "/api/videos/available-folders",
            get(available_folders_handler),
        )
        .route("/api/videos/{id}", put(update_video_handler))
        .route("/api/videos/{id}", delete(delete_video_handler))
}

// -------------------------------
// MediaMTX Authentication Endpoints
// -------------------------------

// Validate stream publisher (called by MediaMTX via runOnInit)
#[tracing::instrument(skip(params, state))]
async fn validate_stream_handler(
    State(state): State<Arc<VideoManagerState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;

    if *token == state.rtmp_publish_token {
        println!("✅ Stream publisher authorized");
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
) -> Result<Html<String>, StatusCode> {
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
    let videos = get_videos(state.repo.as_ref(), user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(
        count = videos.len(),
        authenticated = authenticated,
        "Videos loaded"
    );

    let page_title = if authenticated {
        "🎥 Videos".to_string()
    } else {
        "🎥 Public Videos".to_string()
    };
    let page_subtitle = if authenticated {
        "Browse your entire video collection".to_string()
    } else {
        "Discover our public video content".to_string()
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

    Ok(Html(
        VideoListTemplate {
            authenticated,
            page_title,
            page_subtitle,
            public_videos,
            private_videos,
        }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

#[derive(Deserialize)]
pub struct AccessCodeQuery {
    code: Option<String>,
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
) -> Result<Html<String>, Response> {
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
    let video = state
        .repo
        .get_video_for_player(&slug)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;

    let video = video.ok_or_else(|| {
        let html = NotFoundTemplate { authenticated }
            .render()
            .unwrap_or_default();
        (StatusCode::NOT_FOUND, Html(html)).into_response()
    })?;
    let video_id = video.id;
    let title = video.title;
    let is_public = video.is_public == 1;

    // Build access context for modern access control
    let mut context = AccessContext::new(ResourceType::Video, video_id);
    if let Some(uid) = user_id {
        context = context.with_user(uid);
    }
    if let Some(key) = query.code.clone() {
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
        let html = UnauthorizedTemplate {
            authenticated: false,
        }
        .render()
        .unwrap_or_default();
        return Err((StatusCode::UNAUTHORIZED, Html(html)).into_response());
    }

    // Log successful access with layer information
    info!(
        video_slug = %slug,
        access_layer = ?decision.layer,
        reason = %decision.reason,
        "Access granted to video"
    );

    Ok(Html(
        VideoPlayerTemplate {
            authenticated,
            title,
            slug,
            is_public,
        }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?,
    ))
}

// -------------------------------
// Live Stream Test Handler
// -------------------------------

#[tracing::instrument(skip(session))]
pub async fn live_test_handler(session: Session) -> Result<Html<String>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let template = LiveTestTemplate { authenticated };
    Ok(Html(template.render().unwrap()))
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
    // DB lookup for regular videos - get id, user_id, vault_id, and is_public
    let video = state
        .repo
        .get_video_for_hls(slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let video_id = video.id;
    let _owner_user_id = video.user_id;
    let vault_id = video.vault_id;
    let _is_public = video.is_public == 1;

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
    if let Some(key) = query.code.clone() {
        context = context.with_key(key);
    }

    // Check access using the 4-layer access control system
    let decision = state
        .access_control
        .check_access(context, Permission::Read)
        .await
        .map_err(|e| {
            info!(error = ?e, "Access control error for HLS stream");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !decision.granted {
        // Fallback: workspace folder access code whose workspace owner also owns this vault
        let vault_ok = if let (Some(ref code), Some(ref vid)) = (query.code.as_ref(), vault_id.as_ref()) {
            state
                .repo
                .workspace_folder_code_grants_vault_via_owner(code, vid)
                .await
                .unwrap_or(false)
        } else {
            false
        };

        if !vault_ok {
            info!(
                video_slug = %slug,
                file_path = %file_path,
                reason = %decision.reason,
                "Access denied to HLS stream"
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    info!(
        video_slug = %slug,
        access_layer = ?decision.layer,
        "Access granted to HLS stream"
    );

    // Phase 4.5: Serve VOD file from vault-based storage
    // Use multi-location fallback to find HLS files in new nested structure
    // All media should have vault_id now
    let vault_id = vault_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let hls_file = format!("{}/{}", slug, file_path);
    let full_path = state
        .storage_config
        .user_storage
        .find_media_file(
            &vault_id,
            common::storage::MediaType::Video,
            &hls_file,
        )
        .ok_or_else(|| {
            info!("HLS file not found: {}/{}", slug, file_path);
            StatusCode::NOT_FOUND
        })?;

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
    repo: &dyn MediaRepository,
    code: &str,
    media_type: &str,
    media_slug: &str,
) -> bool {
    let has_access = repo
        .check_access_code_for_media(code, media_type, media_slug)
        .await
        .unwrap_or(false);
    if has_access {
        info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
    }
    has_access
}

/// GET /api/videos - List user's videos for access code selection
#[tracing::instrument(skip(state, session))]
pub async fn list_videos_api_handler(
    State(state): State<Arc<VideoManagerState>>,
    session: Session,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Get user_id from session
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();

    if user_id.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let uid = user_id.unwrap();

    // Fetch user's videos with tags via repository
    let videos = state
        .repo
        .list_user_videos_api(&uid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = videos
        .into_iter()
        .map(|row| {
            let tags: Vec<String> = row
                .tags
                .map(|s| s.split(',').map(|t| t.to_string()).collect())
                .unwrap_or_default();

            serde_json::json!({
                "id": row.id,
                "slug": row.slug,
                "title": row.title,
                "description": row.description,
                "poster_url": row.poster_url,
                "thumbnail_url": row.thumbnail_url,
                "created_at": row.created_at,
                "tags": tags,
                "type": "video"
            })
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_videos(
    repo: &dyn MediaRepository,
    user_id: Option<String>,
) -> Result<Vec<(String, String, i32)>, db::DbError> {
    let rows = repo
        .list_videos_for_page(user_id.as_deref())
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.slug, r.title, r.is_public))
        .collect())
}

// -------------------------------
// Video New Page Handler (Register Video)
// -------------------------------

#[tracing::instrument(skip(session))]
pub async fn video_new_page_handler(
    session: Session,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    let template = VideoNewTemplate { authenticated };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Video Upload Page Handler
// -------------------------------

#[tracing::instrument(skip(session))]
pub async fn video_upload_page_handler(
    session: Session,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    let template = SimpleVideoUploadTemplate { authenticated };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Video Upload Handler (API)
// -------------------------------

#[tracing::instrument(skip(session, state, multipart))]
pub async fn video_upload_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
    multipart: Multipart,
) -> Result<Json<upload::UploadResponse>, (StatusCode, Json<upload::UploadErrorResponse>)> {
    // Create upload state from video manager state
    let upload_state = Arc::new(UploadState::new(
        state.pool.clone(),
        state.storage_config.clone(),
        state.ffmpeg_config.clone(),
        state.hls_config.clone(),
        state.progress_tracker.clone(),
        state.metrics_store.clone(),
        state.audit_logger.clone(),
        state.repo.clone(),
        state.vault_repo.clone(),
    ));

    // Call the upload handler
    handle_video_upload(session, State(upload_state), multipart).await
}

// -------------------------------
// Upload Progress API Handler
// -------------------------------

#[tracing::instrument(skip(state))]
pub async fn get_upload_progress_handler(
    Path(upload_id): Path<String>,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<Json<UploadProgress>, StatusCode> {
    // Get progress from tracker
    match state.progress_tracker.get(&upload_id) {
        Some(progress) => Ok(Json(progress)),
        None => {
            // Check if video exists in database (might be old/completed)
            // Note: media_items doesn't have upload_id, processing_status, processing_progress fields
            // These are tracked in the progress_tracker in memory
            match sqlx::query("SELECT slug FROM media_items WHERE media_type = 'video' LIMIT 0")
                .fetch_optional(&state.pool)
                .await
            {
                Ok(Some(row)) => {
                    // Return progress from database
                    let processing_status: Option<String> = row.try_get("processing_status").ok();
                    let status = match processing_status.as_deref() {
                        Some("complete") => crate::progress::ProgressStatus::Complete,
                        Some("error") => crate::progress::ProgressStatus::Error,
                        _ => crate::progress::ProgressStatus::Processing,
                    };

                    let slug: String = row.try_get("slug").unwrap_or_default();
                    let processing_progress: Option<i32> = row.try_get("processing_progress").ok();

                    Ok(Json(UploadProgress {
                        upload_id: upload_id.clone(),
                        slug,
                        status,
                        progress: processing_progress.unwrap_or(0) as u8,
                        stage: "See database for details".to_string(),
                        started_at: 0,
                        completed_at: None,
                        estimated_completion: None,
                        error: None,
                        metadata: None,
                    }))
                }
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// -------------------------------
// Metrics API Handler
// -------------------------------

/// GET /api/videos/metrics - Get processing metrics
#[tracing::instrument(skip(state))]
pub async fn get_metrics_handler(
    State(state): State<Arc<VideoManagerState>>,
) -> Json<metrics::MetricsSummary> {
    let metrics = state.metrics_store.read().await;
    Json(metrics.summary())
}

/// GET /api/videos/metrics/detailed - Get detailed metrics
#[tracing::instrument(skip(state))]
pub async fn get_detailed_metrics_handler(
    State(state): State<Arc<VideoManagerState>>,
) -> Json<metrics::ProcessingMetrics> {
    let metrics = state.metrics_store.read().await;
    Json(metrics.clone())
}

// -------------------------------
// Available Folders Handler
// -------------------------------

#[derive(Debug, Serialize)]
pub struct FolderInfo {
    name: String,
    has_playlist: bool,
    has_poster: bool,
    segment_count: usize,
}

/// GET /api/videos/available-folders - List video folders on disk without DB entries
#[tracing::instrument(skip(session, state))]
pub async fn available_folders_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<Json<Vec<FolderInfo>>, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let videos_dir = state.storage_dir.join("videos");

    // Read directories from disk
    let mut entries = match tokio::fs::read_dir(&videos_dir).await {
        Ok(entries) => entries,
        Err(_) => return Ok(Json(vec![])),
    };

    let mut folders = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(file_type) = entry.file_type().await {
            if file_type.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    folders.push(name.to_string());
                }
            }
        }
    }

    // Get slugs already registered in DB
    let registered = state
        .repo
        .get_all_video_slugs()
        .await
        .unwrap_or_default();
    let registered_slugs: Vec<&str> = registered.iter().map(|s| s.as_str()).collect();

    // Filter to unregistered folders and gather info
    let mut result = Vec::new();
    for folder in folders {
        if registered_slugs.contains(&folder.as_str()) {
            continue;
        }

        let folder_path = videos_dir.join(&folder);
        let has_playlist = folder_path.join("master.m3u8").exists();
        let has_poster = folder_path.join("thumbnail.webp").exists();

        // Count segments
        let mut segment_count = 0;
        let segments_dir = folder_path.join("segments");
        if segments_dir.exists() {
            if let Ok(mut seg_entries) = tokio::fs::read_dir(&segments_dir).await {
                while let Ok(Some(seg)) = seg_entries.next_entry().await {
                    if let Some(name) = seg.file_name().to_str() {
                        if name.ends_with(".ts") {
                            segment_count += 1;
                        }
                    }
                }
            }
        }

        result.push(FolderInfo {
            name: folder,
            has_playlist,
            has_poster,
            segment_count,
        });
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(result))
}

// -------------------------------
// Register Video Handler (Create DB entry)
// -------------------------------

#[derive(Debug, Deserialize)]
pub struct RegisterVideoRequest {
    slug: String,
    title: String,
    description: Option<String>,
    #[serde(rename = "isPublic")]
    is_public: Option<bool>,
    #[serde(rename = "groupId")]
    group_id: Option<String>,
}

/// POST /api/videos - Register a video folder as a DB entry
#[tracing::instrument(skip(session, state))]
pub async fn register_video_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
    Json(req): Json<RegisterVideoRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    let user_id: Option<String> = session.get::<String>("user_id").await.ok().flatten();
    let user_id = user_id.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "No user_id in session".to_string(),
        )
    })?;

    // Validate slug
    let slug = req.slug.trim().to_string();
    if slug.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Slug is required".to_string()));
    }

    // Validate folder exists on disk
    let folder_path = state.storage_dir.join("videos").join(&slug);
    if !folder_path.exists() || !folder_path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Video folder '{}' does not exist on disk", slug),
        ));
    }

    // Validate master.m3u8 exists
    if !folder_path.join("master.m3u8").exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Video folder '{}' does not contain master.m3u8", slug),
        ));
    }

    // Validate title
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Title is required".to_string()));
    }

    let is_public = req.is_public.unwrap_or(false);
    let description = req.description.unwrap_or_default();

    // Parse group_id
    let group_id: Option<i32> = req.group_id.as_ref().and_then(|g| {
        if g.is_empty() {
            None
        } else {
            g.parse::<i32>().ok()
        }
    });

    // Insert into database
    let result = sqlx::query(
        "INSERT INTO videos (slug, title, description, is_public, user_id, group_id, status, upload_date)
         VALUES (?, ?, ?, ?, ?, ?, 'active', CURRENT_TIMESTAMP)"
    )
    .bind(&slug)
    .bind(&title)
    .bind(&description)
    .bind(if is_public { 1i32 } else { 0i32 })
    .bind(&user_id)
    .bind(group_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint") {
            (StatusCode::CONFLICT, format!("Video with slug '{}' already exists", slug))
        } else {
            tracing::error!("Database error registering video: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
        }
    })?;

    let video_id = result.last_insert_rowid();

    info!(
        video_id = video_id,
        slug = %slug,
        title = %title,
        "Video registered successfully"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "id": video_id,
        "slug": slug,
        "title": title,
        "message": "Video registered successfully"
    })))
}

// -------------------------------
// Video Edit Page Handler
// -------------------------------

fn video_detail_from_row(row: &sqlx::sqlite::SqliteRow) -> VideoDetail {
    let group_id: Option<i32> = row.try_get("group_id").ok();
    VideoDetail {
        id: row.try_get("id").unwrap_or(0),
        slug: row.try_get("slug").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").ok(),
        short_description: row.try_get("short_description").ok(),
        is_public: row.try_get::<i32, _>("is_public").unwrap_or(0) == 1,
        user_id: row.try_get("user_id").ok(),
        group_id,
        group_id_str: group_id.map(|id| id.to_string()).unwrap_or_default(),
        duration: row.try_get("duration").ok(),
        file_size: row.try_get("file_size").ok(),
        resolution: row.try_get("resolution").ok(),
        width: row.try_get("width").ok(),
        height: row.try_get("height").ok(),
        fps: row.try_get("fps").ok(),
        codec: row.try_get("codec").ok(),
        thumbnail_url: row.try_get("thumbnail_url").ok(),
        poster_url: row.try_get("poster_url").ok(),
        category: row.try_get("category").ok(),
        language: row.try_get("language").ok(),
        status: row
            .try_get("status")
            .unwrap_or_else(|_| "active".to_string()),
        featured: row.try_get::<i32, _>("featured").unwrap_or(0) == 1,
        allow_comments: row.try_get::<i32, _>("allow_comments").unwrap_or(1) == 1,
        allow_download: row.try_get::<i32, _>("allow_download").unwrap_or(0) == 1,
        mature_content: row.try_get::<i32, _>("mature_content").unwrap_or(0) == 1,
        view_count: row.try_get("view_count").unwrap_or(0),
        like_count: row.try_get("like_count").unwrap_or(0),
        download_count: row.try_get("download_count").unwrap_or(0),
        share_count: row.try_get("share_count").unwrap_or(0),
        upload_date: row.try_get("upload_date").unwrap_or_default(),
        seo_title: row.try_get("seo_title").ok(),
        seo_description: row.try_get("seo_description").ok(),
        seo_keywords: row.try_get("seo_keywords").ok(),
    }
}

fn video_detail_from_media_item(row: &db::media::MediaItemRow) -> VideoDetail {
    VideoDetail {
        id: row.id as i64,
        slug: row.slug.clone(),
        title: row.title.clone(),
        description: row.description.clone(),
        short_description: None,
        is_public: row.is_public == 1,
        user_id: row.user_id.clone(),
        group_id: row.group_id,
        group_id_str: row.group_id.map(|id| id.to_string()).unwrap_or_default(),
        duration: None,
        file_size: Some(row.file_size),
        resolution: None,
        width: None,
        height: None,
        fps: None,
        codec: None,
        thumbnail_url: row.thumbnail_url.clone(),
        poster_url: row.thumbnail_url.clone(),
        category: row.category.clone(),
        language: None,
        status: row.status.clone(),
        featured: row.featured == 1,
        allow_comments: row.allow_comments == 1,
        allow_download: row.allow_download == 1,
        mature_content: row.mature_content == 1,
        view_count: row.view_count as i64,
        like_count: row.like_count as i64,
        download_count: row.download_count as i64,
        share_count: row.share_count as i64,
        upload_date: row.created_at.clone(),
        seo_title: row.seo_title.clone(),
        seo_description: row.seo_description.clone(),
        seo_keywords: row.seo_keywords.clone(),
    }
}

/// GET /videos/:slug/edit - Serve the video edit page
#[tracing::instrument(skip(session, state))]
pub async fn video_edit_page_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    // Fetch video from database
    let media_row = state
        .repo
        .get_media_by_slug(&slug)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching video: {}", e);
            (StatusCode::NOT_FOUND, format!("Video not found: {}", e))
        })?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, format!("Video not found: {}", slug))
        })?;

    let video = video_detail_from_media_item(&media_row);

    let template = VideoEditTemplate {
        authenticated,
        video,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Update Video Handler (API)
// -------------------------------

#[derive(Debug, Deserialize)]
pub struct UpdateVideoRequest {
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "shortDescription")]
    short_description: Option<String>,
    #[serde(rename = "isPublic")]
    is_public: Option<bool>,
    category: Option<String>,
    language: Option<String>,
    status: Option<String>,
    featured: Option<bool>,
    #[serde(rename = "allowComments")]
    allow_comments: Option<bool>,
    #[serde(rename = "allowDownload")]
    allow_download: Option<bool>,
    #[serde(rename = "matureContent")]
    mature_content: Option<bool>,
    #[serde(rename = "seoTitle")]
    seo_title: Option<String>,
    #[serde(rename = "seoDescription")]
    seo_description: Option<String>,
    #[serde(rename = "seoKeywords")]
    seo_keywords: Option<String>,
    #[serde(rename = "groupId")]
    group_id: Option<serde_json::Value>,
}

/// PUT /api/videos/:id - Update video metadata
#[tracing::instrument(skip(session, state))]
pub async fn update_video_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
    Path(id): Path<i64>,
    Json(update_req): Json<UpdateVideoRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("Update video handler called for video_id={}", id);

    // Get authenticated user
    let user_sub = get_user_from_session(&session, state.repo.as_ref())
        .await
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            )
        })?;

    // Check if user can modify this video
    let can_modify = can_modify_video(&state.access_control, id as i32, &user_sub)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Access check failed: {}", e),
            )
        })?;

    if !can_modify {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have permission to edit this video".to_string(),
        ));
    }

    // Build dynamic UPDATE query
    #[derive(Debug)]
    enum ParamValue {
        Text(String),
        Bool(bool),
        OptionalInt(Option<i32>),
    }

    let mut updates = Vec::new();
    let mut param_values: Vec<ParamValue> = Vec::new();

    if let Some(title) = &update_req.title {
        updates.push("title = ?");
        param_values.push(ParamValue::Text(title.clone()));
    }
    if let Some(description) = &update_req.description {
        updates.push("description = ?");
        param_values.push(ParamValue::Text(description.clone()));
    }
    if let Some(short_description) = &update_req.short_description {
        updates.push("short_description = ?");
        param_values.push(ParamValue::Text(short_description.clone()));
    }
    if let Some(is_public) = update_req.is_public {
        updates.push("is_public = ?");
        param_values.push(ParamValue::Bool(is_public));
    }
    if let Some(category) = &update_req.category {
        updates.push("category = ?");
        param_values.push(ParamValue::Text(category.clone()));
    }
    if let Some(language) = &update_req.language {
        updates.push("language = ?");
        param_values.push(ParamValue::Text(language.clone()));
    }
    if let Some(status) = &update_req.status {
        updates.push("status = ?");
        param_values.push(ParamValue::Text(status.clone()));
    }
    if let Some(featured) = update_req.featured {
        updates.push("featured = ?");
        param_values.push(ParamValue::Bool(featured));
    }
    if let Some(allow_comments) = update_req.allow_comments {
        updates.push("allow_comments = ?");
        param_values.push(ParamValue::Bool(allow_comments));
    }
    if let Some(allow_download) = update_req.allow_download {
        updates.push("allow_download = ?");
        param_values.push(ParamValue::Bool(allow_download));
    }
    if let Some(mature_content) = update_req.mature_content {
        updates.push("mature_content = ?");
        param_values.push(ParamValue::Bool(mature_content));
    }
    if let Some(seo_title) = &update_req.seo_title {
        updates.push("seo_title = ?");
        param_values.push(ParamValue::Text(seo_title.clone()));
    }
    if let Some(seo_description) = &update_req.seo_description {
        updates.push("seo_description = ?");
        param_values.push(ParamValue::Text(seo_description.clone()));
    }
    if let Some(seo_keywords) = &update_req.seo_keywords {
        updates.push("seo_keywords = ?");
        param_values.push(ParamValue::Text(seo_keywords.clone()));
    }
    // Handle group_id - can be number, string, or empty/null
    if let Some(group_id_val) = &update_req.group_id {
        updates.push("group_id = ?");
        let parsed = match group_id_val {
            serde_json::Value::Number(n) => n.as_i64().map(|v| v as i32),
            serde_json::Value::String(s) => {
                if s.is_empty() {
                    None
                } else {
                    s.parse::<i32>().ok()
                }
            }
            serde_json::Value::Null => None,
            _ => None,
        };
        param_values.push(ParamValue::OptionalInt(parsed));
    }

    if updates.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let sql = format!(
        "UPDATE videos SET {}, last_modified = CURRENT_TIMESTAMP WHERE id = ?",
        updates.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for param in param_values {
        query = match param {
            ParamValue::Text(s) => query.bind(s),
            ParamValue::Bool(b) => query.bind(if b { 1i32 } else { 0i32 }),
            ParamValue::OptionalInt(opt) => query.bind(opt),
        };
    }
    query = query.bind(id);

    match query.execute(&state.pool).await {
        Ok(result) => {
            tracing::info!(
                "Video {} updated successfully. Rows affected: {:?}",
                id,
                result.rows_affected()
            );

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Video updated successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Database error updating video {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ))
        }
    }
}

// -------------------------------
// Delete Video Handler (API)
// -------------------------------

/// DELETE /api/videos/:id - Delete video DB entry and physical files
#[tracing::instrument(skip(session, state))]
pub async fn delete_video_handler(
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if !authenticated {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    let user_id: Option<String> = session.get::<String>("user_id").await.ok().flatten();

    // Check if user is emergency admin (bypass access control for superuser)
    let is_emergency_admin = user_id
        .as_ref()
        .map(|uid| uid.starts_with("emergency-"))
        .unwrap_or(false);

    if !is_emergency_admin {
        // Check access with Delete permission
        let mut context = AccessContext::new(ResourceType::Video, id as i32);
        if let Some(uid) = &user_id {
            context = context.with_user(uid.clone());
        }

        let decision = state
            .access_control
            .check_access(context, Permission::Delete)
            .await
            .map_err(|e| {
                info!(error = ?e, "Access control error for video deletion");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Access check failed".to_string(),
                )
            })?;

        if !decision.granted {
            info!(video_id = id, reason = %decision.reason, "Access denied to delete video");
            return Err((
                StatusCode::FORBIDDEN,
                "Cannot delete this video".to_string(),
            ));
        }
    } else {
        info!(
            video_id = id,
            user_id = ?user_id,
            "Emergency admin bypassing access control for video deletion"
        );
    }

    // Get video details before deletion to locate physical files
    let video_info = state
        .repo
        .get_video_for_deletion(id)
        .await
        .ok()
        .flatten();

    info!(
        video_id = id,
        video_info = ?video_info,
        "Retrieved video info for deletion"
    );

    // Delete associated tags
    let _ = state.repo.delete_video_tags(id).await;

    // Delete associated access permissions
    let _ = state.repo.delete_video_permissions(id as i32).await;

    // Delete the video record
    state
        .repo
        .delete_video_by_id(id)
        .await
        .map_err(|e| {
            tracing::error!("Database error deleting video {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    // Delete physical files if video info was retrieved
    if let Some((filename, vault_id_opt)) = video_info {
        if let Some(vault_id) = vault_id_opt {
            // Video files are stored in: storage/vaults/{vault_id}/videos/{filename}/
            let video_dir = std::path::PathBuf::from("storage")
                .join("vaults")
                .join(&vault_id)
                .join("videos")
                .join(&filename);

            info!(
                video_id = id,
                vault_id = %vault_id,
                filename = %filename,
                video_dir = ?video_dir,
                exists = video_dir.exists(),
                "Attempting to delete video directory"
            );

            if video_dir.exists() {
                match tokio::fs::remove_dir_all(&video_dir).await {
                    Ok(_) => {
                        info!(path = ?video_dir, "Video directory deleted successfully");
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, path = ?video_dir, "Failed to delete video directory");
                    }
                }
            } else {
                info!(path = ?video_dir, "Video directory does not exist at expected path");
            }

            // Also try to delete video thumbnail if it exists
            let thumb_path = std::path::PathBuf::from("storage")
                .join("vaults")
                .join(&vault_id)
                .join("thumbnails")
                .join("videos")
                .join(format!("{}.jpg", &filename));

            if thumb_path.exists() {
                let _ = tokio::fs::remove_file(&thumb_path).await;
                info!(path = ?thumb_path, "Video thumbnail deleted");
            }
        } else {
            info!(
                video_id = id,
                filename = %filename,
                "No vault_id found for video"
            );
        }
    } else {
        info!(video_id = id, "No video info found for deletion");
    }

    info!(
        video_id = id,
        "Video deleted successfully (including physical files)"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Video deleted successfully."
    })))
}

/// Helper function to check if user can modify video
/// Uses modern AccessControlService with Edit permission
async fn can_modify_video(
    access_control: &AccessControlService,
    video_id: i32,
    user_sub: &str,
) -> Result<bool, sqlx::Error> {
    let context = AccessContext::new(ResourceType::Video, video_id).with_user(user_sub.to_string());

    match access_control.check_access(context, Permission::Edit).await {
        Ok(decision) => Ok(decision.granted),
        Err(_) => Ok(false),
    }
}

/// Helper to get user from session
async fn get_user_from_session(session: &Session, repo: &dyn MediaRepository) -> Option<String> {
    tracing::debug!("get_user_from_session: Attempting to get user_id from session");
    let user_id: Option<String> = session.get("user_id").await.ok().flatten();
    tracing::debug!(
        "get_user_from_session: user_id from session = {:?}",
        user_id
    );

    if let Some(id) = user_id {
        tracing::debug!(
            "get_user_from_session: Verifying user exists with id = {}",
            id
        );
        // Verify user exists
        let exists = repo.user_exists(&id).await.unwrap_or(false);

        tracing::debug!(
            "get_user_from_session: User verification result = {:?}",
            exists
        );
        if exists { Some(id) } else { None }
    } else {
        tracing::warn!("get_user_from_session: No user_id found in session!");
        None
    }
}

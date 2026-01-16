use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tokio_util::io::ReaderStream;
use tower_sessions::Session;

// -------------------------------
// Template Structs
// -------------------------------

#[derive(Template)]
#[template(path = "videos/list.html")]
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
}

impl VideoManagerState {
    pub fn new(pool: Pool<Sqlite>, storage_dir: PathBuf, http_client: Client) -> Self {
        Self {
            pool,
            storage_dir,
            http_client,
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
}

// -------------------------------
// MediaMTX Authentication Endpoints
// -------------------------------

// Validate stream publisher (called by MediaMTX via runOnInit)
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

    // Lookup video in database
    let video: Option<(String, i32)> =
        sqlx::query_as("SELECT title, is_public FROM videos WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;

    let (title, is_public) = video.ok_or_else(|| {
        (StatusCode::NOT_FOUND, NotFoundTemplate { authenticated }).into_response()
    })?;
    let is_public = is_public == 1;

    // For private videos, check authentication or access code
    if !is_public && !authenticated {
        // Check if access code is provided and valid
        if let Some(code) = &query.access_code {
            if !check_access_code(&state.pool, code, "video", &slug).await {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    UnauthorizedTemplate {
                        authenticated: false,
                    },
                )
                    .into_response());
            }
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                UnauthorizedTemplate {
                    authenticated: false,
                },
            )
                .into_response());
        }
    }

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
    // DB lookup for regular videos
    let video: Option<(i32,)> = sqlx::query_as("SELECT is_public FROM videos WHERE slug = ?")
        .bind(slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_public = video.map(|(p,)| p == 1).unwrap_or(false);

    // Authentication required for non-public videos
    if !is_public {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            // Check if access code is provided and valid
            if let Some(code) = &query.access_code {
                if !check_access_code(&state.pool, code, "video", slug).await {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Serve VOD file from storage
    let base_folder = if is_public {
        "videos/public"
    } else {
        "videos/private"
    };
    let full_path = state
        .storage_dir
        .join(base_folder)
        .join(slug)
        .join(file_path);

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

        permission.is_some()
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

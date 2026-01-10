use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use sqlx::{Pool, Sqlite};
use std::{path::PathBuf, sync::Arc};
use tokio_util::io::ReaderStream;
use tower_sessions::Session;

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
) -> Result<Html<String>, StatusCode> {
    // Check if user is authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get videos based on authentication status
    let videos = get_videos(&state.pool, authenticated)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page_title = if authenticated {
        "🎥 All Videos"
    } else {
        "🎥 Public Videos"
    };

    let mut html = format!(
        r#"<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #4CAF50; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }}
        .auth-links {{ display: flex; gap: 15px; }}
        .auth-links a {{ color: #1976D2; text-decoration: none; font-weight: bold; }}
        .auth-links a:hover {{ text-decoration: underline; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 10px; margin: 5px 0; background: #f5f5f5; border-radius: 4px; display: flex; justify-content: space-between; align-items: center; }}
        li.private {{ background: #fff3e0; border-left: 4px solid #ff9800; }}
        a {{ text-decoration: none; color: #333; font-weight: bold; }}
        a:hover {{ color: #4CAF50; }}
        .badge {{
            background: #ff9800;
            color: white;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: bold;
        }}
        .info {{ background: #e3f2fd; padding: 15px; border-radius: 4px; margin: 20px 0; }}
        .section {{ margin: 30px 0; }}
        .section h2 {{ color: #666; font-size: 18px; margin-bottom: 10px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{}</h1>
        <div class="auth-links">
            <a href="/">🏠 Home</a>
            <a href="/images">📸 Images</a>"#,
        page_title, page_title
    );

    if authenticated {
        html.push_str("<a href=\"/logout\">Logout</a>");
    } else {
        html.push_str("<a href=\"/login\">Login</a>");
    }

    html.push_str("</div></div>");

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

    // Show public videos
    if !public_videos.is_empty() {
        html.push_str("<div class=\"section\"><h2>📺 Public Videos</h2><ul>");
        for (slug, title, _) in public_videos {
            html.push_str(&format!(
                "<li><a href=\"/watch/{}\">{}</a></li>",
                slug, title
            ));
        }
        html.push_str("</ul></div>");
    } else {
        html.push_str("<div class=\"section\"><p>No public videos available.</p></div>");
    }

    // Show private videos (only if authenticated)
    if authenticated {
        if !private_videos.is_empty() {
            html.push_str("<div class=\"section\"><h2>🔒 Private Videos</h2><ul>");
            for (slug, title, _) in private_videos {
                html.push_str(&format!(
                    "<li class=\"private\"><a href=\"/watch/{}\">{}</a><span class=\"badge\">PRIVATE</span></li>",
                    slug, title
                ));
            }
            html.push_str("</ul></div>");
        }
    }

    // Info section
    if !authenticated {
        html.push_str(
            r#"<div class="info">
            <p><strong>Want to watch private videos and live streams?</strong></p>
            <p><a href="/login">Login</a> to access exclusive content</p>
            <p><a href="/test">Test Live Stream Player</a></p>
        </div>"#,
        );
    } else {
        html.push_str(
            r#"<div class="info">
            <p><a href="/test">📡 Test Live Stream Player</a></p>
            <p><a href="/upload">📤 Upload Images</a></p>
        </div>"#,
        );
    }

    html.push_str("</body></html>");

    Ok(Html(html))
}

// -------------------------------
// Video Player Page Handler
// -------------------------------

pub async fn video_player_handler(
    Path(slug): Path<String>,
    session: Session,
    State(state): State<Arc<VideoManagerState>>,
) -> Result<Html<String>, StatusCode> {
    // Lookup video in database
    let video: Option<(String, i32)> = sqlx::query_as(
        "SELECT title, is_public FROM videos WHERE slug = ?"
    )
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (title, is_public) = video.ok_or(StatusCode::NOT_FOUND)?;
    let is_public = is_public == 1;

    // Check authentication for private videos
    if !is_public {
        let authenticated: bool = session
            .get("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);

        if !authenticated {
            return Ok(Html(String::from(
                r#"<html>
<head>
    <title>Private Video</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 100px auto; padding: 20px; text-align: center; }
        .error { background: #ffebee; padding: 20px; border-radius: 8px; color: #c62828; }
        a { color: #1976D2; text-decoration: none; font-weight: bold; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="error">
        <h1>🔒 Authentication Required</h1>
        <p>This is a private video. Please login to watch.</p>
        <p><a href="/login">Login</a> | <a href="/">Back to Home</a></p>
    </div>
</body>
</html>"#,
            )));
        }
    }

    // Generate video player HTML with HLS.js support
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - Video Player</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: Arial, sans-serif;
            background: #000;
            color: #fff;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            background: #1a1a1a;
            padding: 15px 20px;
            margin-bottom: 20px;
            border-radius: 8px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        .back-link {{
            color: #4CAF50;
            text-decoration: none;
            font-weight: bold;
        }}
        .back-link:hover {{ text-decoration: underline; }}
        h1 {{
            font-size: 24px;
            margin: 10px 0;
        }}
        .video-wrapper {{
            position: relative;
            width: 100%;
            padding-bottom: 56.25%; /* 16:9 aspect ratio */
            background: #000;
            border-radius: 8px;
            overflow: hidden;
        }}
        video {{
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
        }}
        .controls {{
            margin-top: 20px;
            background: #1a1a1a;
            padding: 15px;
            border-radius: 8px;
        }}
        .info {{
            color: #888;
            font-size: 14px;
        }}
        .error {{
            background: #f44336;
            color: white;
            padding: 15px;
            border-radius: 8px;
            margin: 20px 0;
        }}
        @media (max-width: 768px) {{
            h1 {{ font-size: 18px; }}
            .header {{ flex-direction: column; gap: 10px; }}
        }}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/hls.js@latest"></script>
</head>
<body>
    <div class="container">
        <div class="header">
            <div>
                <a href="/" class="back-link">🏠 Home</a> |
                <a href="/videos" class="back-link">📺 Videos</a>
            </div>
            <div class="info" id="player-info">Initializing player...</div>
        </div>

        <h1>{}</h1>

        <div class="video-wrapper">
            <video id="video" controls autoplay></video>
        </div>

        <div class="controls" id="error-message" style="display: none;">
            <div class="error">
                <strong>Error:</strong> <span id="error-text"></span>
            </div>
        </div>
    </div>

    <script>
        const video = document.getElementById('video');
        const videoSrc = '/hls/{}/master.m3u8';
        const playerInfo = document.getElementById('player-info');
        const errorMessage = document.getElementById('error-message');
        const errorText = document.getElementById('error-text');

        function showError(message) {{
            errorMessage.style.display = 'block';
            errorText.textContent = message;
            playerInfo.textContent = 'Playback failed';
        }}

        if (video.canPlayType('application/vnd.apple.mpegurl')) {{
            video.src = videoSrc;
            playerInfo.textContent = 'Using native HLS player (Safari)';

            video.addEventListener('error', function() {{
                showError('Failed to load video. Please check your connection.');
            }});
        }}
        else if (Hls.isSupported()) {{
            const hls = new Hls({{
                enableWorker: true,
                lowLatencyMode: false,
                backBufferLength: 90
            }});

            hls.loadSource(videoSrc);
            hls.attachMedia(video);

            hls.on(Hls.Events.MANIFEST_PARSED, function() {{
                playerInfo.textContent = 'Using HLS.js player (Firefox, Chrome, Edge)';
                video.play().catch(e => {{
                    console.log('Autoplay prevented:', e);
                    playerInfo.textContent = 'Click play to start';
                }});
            }});

            hls.on(Hls.Events.ERROR, function(event, data) {{
                console.error('HLS.js error:', data);
                if (data.fatal) {{
                    switch(data.type) {{
                        case Hls.ErrorTypes.NETWORK_ERROR:
                            showError('Network error - trying to recover...');
                            hls.startLoad();
                            break;
                        case Hls.ErrorTypes.MEDIA_ERROR:
                            showError('Media error - trying to recover...');
                            hls.recoverMediaError();
                            break;
                        default:
                            showError('Fatal error: ' + data.details);
                            hls.destroy();
                            break;
                    }}
                }}
            }});
        }}
        else {{
            showError('Your browser does not support HLS video streaming. Please use a modern browser like Chrome, Firefox, Safari, or Edge.');
            playerInfo.textContent = 'Unsupported browser';
        }}

        document.addEventListener('keydown', function(e) {{
            if (e.key === ' ' || e.key === 'Spacebar') {{
                e.preventDefault();
                if (video.paused) {{
                    video.play();
                }} else {{
                    video.pause();
                }}
            }}
            if (e.key === 'ArrowLeft') {{
                video.currentTime -= 10;
            }}
            if (e.key === 'ArrowRight') {{
                video.currentTime += 10;
            }}
            if (e.key === 'f' || e.key === 'F') {{
                if (document.fullscreenElement) {{
                    document.exitFullscreen();
                }} else {{
                    video.requestFullscreen();
                }}
            }}
        }});
    </script>
</body>
</html>"#,
        title, title, slug
    );

    Ok(Html(html))
}

// -------------------------------
// HLS Proxy Handler for Live Streams and VOD
// -------------------------------

pub async fn hls_proxy_handler(
    Path(path): Path<String>,
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
            return Err(StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY));
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
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Serve VOD file from storage
    let base_folder = if is_public { "videos/public" } else { "videos/private" };
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

pub async fn mediamtx_status(State(state): State<Arc<VideoManagerState>>) -> Result<String, StatusCode> {
    let url = format!("{}/v3/paths/list", MEDIAMTX_API_URL);

    let response = state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let text = response
        .text()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(text)
}

// -------------------------------
// Helper Functions
// -------------------------------

pub async fn get_videos(
    pool: &Pool<Sqlite>,
    authenticated: bool,
) -> Result<Vec<(String, String, i32)>, sqlx::Error> {
    if authenticated {
        sqlx::query_as("SELECT slug, title, is_public FROM videos ORDER BY is_public DESC, title")
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as("SELECT slug, title, is_public FROM videos WHERE is_public = 1 ORDER BY title")
            .fetch_all(pool)
            .await
    }
}

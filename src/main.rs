use axum::{
    extract::State,
    http::{header::HeaderValue, Method, StatusCode},
    response::Html,
    routing::{get, post},
    Router,
};
use reqwest::Client;
use sqlx::sqlite::SqlitePoolOptions;
use std::{net::SocketAddr, sync::Arc};
use time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, Session, SessionManagerLayer};

// Import the crates
use image_manager::{ImageManagerState, image_routes};
use user_auth::{AuthState, auth_routes};
use video_manager::{VideoManagerState, video_routes, RTMP_PUBLISH_TOKEN, get_videos};

// -------------------------------
// Shared App State
// -------------------------------
#[derive(Clone)]
#[allow(dead_code)]
struct AppState {
    video_state: Arc<VideoManagerState>,
    image_state: Arc<ImageManagerState>,
    auth_state: Arc<AuthState>,
}

// -------------------------------
// Main Page Handler
// -------------------------------

async fn index_handler(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Html<String>, StatusCode> {
    // Check if user is authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    // Get videos based on authentication status
    let videos = get_videos(&state.video_state.pool, authenticated)
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
    }

    // Show private videos (only if authenticated)
    if authenticated && !private_videos.is_empty() {
        html.push_str("<div class=\"section\"><h2>🔒 Private Videos</h2><ul>");
        for (slug, title, _) in private_videos {
            html.push_str(&format!(
                "<li class=\"private\"><a href=\"/watch/{}\">{}</a><span class=\"badge\">PRIVATE</span></li>",
                slug, title
            ));
        }
        html.push_str("</ul></div>");
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
// Test Page Handler
// -------------------------------

async fn test_page_handler() -> Html<&'static str> {
    Html(include_str!("../test-hls.html"))
}

// -------------------------------
// Health Check Endpoint
// -------------------------------

async fn health_check() -> &'static str {
    "OK"
}

// -------------------------------
// Webhook Handlers (Optional)
// -------------------------------

async fn webhook_stream_ready() -> StatusCode {
    println!("📡 Stream is now live!");
    StatusCode::OK
}

async fn webhook_stream_ended() -> StatusCode {
    println!("📡 Stream has ended");
    StatusCode::OK
}

// -------------------------------
// Main Function
// -------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("\n🚀 Initializing Modular Video Server...");

    // DB setup
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:video.db?mode=rwc")
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let storage_dir = std::env::current_dir()?.join("storage");
    std::fs::create_dir_all(&storage_dir)?;

    // Create video storage directories
    std::fs::create_dir_all(storage_dir.join("videos/public"))?;
    std::fs::create_dir_all(storage_dir.join("videos/private"))?;

    // Create image storage directories
    std::fs::create_dir_all(storage_dir.join("images/public"))?;
    std::fs::create_dir_all(storage_dir.join("images/private"))?;

    // Create HTTP client for MediaMTX communication
    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Initialize module states
    let video_state = Arc::new(VideoManagerState::new(
        pool.clone(),
        storage_dir.clone(),
        http_client,
    ));

    let image_state = Arc::new(ImageManagerState::new(pool.clone(), storage_dir));

    let auth_state = Arc::new(AuthState::new());

    let app_state = Arc::new(AppState {
        video_state: video_state.clone(),
        image_state: image_state.clone(),
        auth_state: auth_state.clone(),
    });

    // Session layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax);

    // Build the application router
    let app = Router::new()
        // Main routes
        .route("/", get(index_handler))
        .route("/test", get(test_page_handler))
        .route("/health", get(health_check))
        // Webhook endpoints (optional)
        .route("/api/webhooks/stream-ready", post(webhook_stream_ready))
        .route("/api/webhooks/stream-ended", post(webhook_stream_ended))
        .with_state(app_state)
        // Merge module routers
        .merge(
            auth_routes()
                .with_state(auth_state)
        )
        .merge(
            video_routes()
                .with_state(video_state)
        )
        .merge(
            image_routes()
                .with_state(image_state)
        )
        // Apply middleware
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(tower_http::cors::AllowOrigin::predicate(
                            |origin: &HeaderValue, _| {
                                let origin_str = origin.to_str().unwrap_or("");
                                (origin_str.ends_with("appkask.com")
                                    || origin_str.ends_with(".appkask.com"))
                                    || origin_str.contains("localhost")
                            },
                        ))
                        .allow_credentials(true)
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                        .allow_headers([
                            axum::http::header::CONTENT_TYPE,
                            axum::http::header::RANGE,
                            axum::http::header::AUTHORIZATION,
                        ]),
                )
                .layer(session_layer),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   🎥  MODULAR VIDEO SERVER - READY!                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("📦 MODULES LOADED:");
    println!("   ✅ video-manager    (Video streaming & HLS proxy)");
    println!("   ✅ image-manager    (Image upload & serving)");
    println!("   ✅ user-auth        (Session management, OIDC ready)");

    println!("\n📊 SERVER ENDPOINTS:");
    println!("   • Web UI:        http://{}", addr);
    println!("   • Test Player:   http://{}/test", addr);
    println!("   • Login:         http://{}/login", addr);
    println!("   • Images:        http://{}/images", addr);
    println!("   • Upload:        http://{}/upload", addr);
    println!("   • Health:        http://{}/health", addr);
    println!("   • MediaMTX API:  http://{}/api/mediamtx/status", addr);

    println!("\n📡 MEDIAMTX CONFIGURATION:");
    println!("   • RTMP Input:    rtmp://localhost:1935/live");
    println!("   • HLS Output:    http://localhost:8888/live/index.m3u8");
    println!("   • WebRTC Output: http://localhost:8889/live");
    println!("   • API:           http://localhost:9997");
    println!("   • Metrics:       http://localhost:9998/metrics");

    println!("\n🎬 STREAMING COMMANDS:");
    println!("\n   macOS (Camera + Microphone):");
    println!("   ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i \"0:0\" \\");
    println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
    println!("     -c:a aac -b:a 128k -ar 44100 \\");
    println!("     -f flv \"rtmp://localhost:1935/live?token={}\"", RTMP_PUBLISH_TOKEN);

    println!("\n   Linux (Webcam + Microphone):");
    println!("   ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 \\");
    println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
    println!("     -c:a aac -b:a 128k -ar 44100 \\");
    println!("     -f flv \"rtmp://localhost:1935/live?token={}\"", RTMP_PUBLISH_TOKEN);

    println!("\n   OBS Studio:");
    println!("   • Server:     rtmp://localhost:1935/live");
    println!("   • Stream Key: ?token={}", RTMP_PUBLISH_TOKEN);

    println!("\n⚠️  IMPORTANT:");
    println!("   1. Make sure MediaMTX is running: mediamtx mediamtx.yml");
    println!("   2. Login first: http://{}/login", addr);
    println!("   3. Then watch: http://{}/test", addr);

    println!("\n💡 TIPS:");
    println!("   • List devices: ffmpeg -f avfoundation -list_devices true -i \"\"");
    println!("   • Check MediaMTX: curl http://localhost:9997/v3/paths/list");
    println!("   • View logs: MediaMTX logs appear in its terminal");

    println!("\n🔧 ARCHITECTURE:");
    println!("   This server is now modular with separate crates:");
    println!("   • crates/video-manager - Video streaming logic");
    println!("   • crates/image-manager - Image handling logic");
    println!("   • crates/user-auth     - Authentication (OIDC ready)");

    println!("\n{}\n", "═".repeat(64));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

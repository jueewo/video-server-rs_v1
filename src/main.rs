use askama::Template;
use axum::{
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
use tower_http::{cors::CorsLayer, services::ServeDir};
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, Session, SessionManagerLayer};

// Import the crates
use image_manager::{image_routes, ImageManagerState};
use user_auth::{auth_routes, AuthState, OidcConfig};
use video_manager::{video_routes, VideoManagerState, RTMP_PUBLISH_TOKEN};

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
// Templates
// -------------------------------

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    authenticated: bool,
}

// -------------------------------
// Main Page Handler
// -------------------------------

async fn index_handler(session: Session) -> Result<Html<String>, StatusCode> {
    // Check if user is authenticated
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    let template = IndexTemplate { authenticated };
    Ok(Html(template.render().unwrap()))
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
    // Load environment variables from .env file (if it exists)
    let _ = dotenvy::dotenv();

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

    let image_state = Arc::new(ImageManagerState::new(pool.clone(), storage_dir.clone()));

    // Initialize OIDC configuration
    let oidc_config = OidcConfig::from_env();
    println!("🔐 OIDC Configuration:");
    println!("   - Issuer URL: {}", oidc_config.issuer_url);
    println!("   - Client ID: {}", oidc_config.client_id);
    println!("   - Redirect URI: {}", oidc_config.redirect_uri);

    let auth_state = match AuthState::new(oidc_config.clone()).await {
        Ok(state) => {
            if state.oidc_client.is_some() {
                println!("✅ OIDC authentication enabled");
            } else {
                println!("⚠️  OIDC authentication disabled (provider unavailable)");
            }
            Arc::new(state)
        }
        Err(e) => {
            println!("⚠️  Failed to initialize OIDC: {}", e);
            println!("   Using emergency login only");
            Arc::new(AuthState::new_without_oidc(oidc_config))
        }
    };

    let app_state = Arc::new(AppState {
        video_state: video_state.clone(),
        image_state: image_state.clone(),
        auth_state: auth_state.clone(),
    });

    // Session layer with explicit configuration
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("video_server_session") // Explicit session cookie name
        .with_secure(false) // Set to true in production with HTTPS
        .with_http_only(true) // Prevent JavaScript access
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax) // Allow cross-site for OIDC redirects
        .with_path("/"); // Cookie available for entire site

    println!("🍪 Session Configuration:");
    println!("   - Cookie name: video_server_session");
    println!("   - HTTP-only: true");
    println!("   - Same-site: Lax");
    println!("   - Expiry: 7 days inactivity");

    // Build the application router
    let app = Router::new()
        // Main routes
        .route("/", get(index_handler))
        .route("/health", get(health_check))
        // Webhook endpoints (optional)
        .route("/api/webhooks/stream-ready", post(webhook_stream_ready))
        .route("/api/webhooks/stream-ended", post(webhook_stream_ended))
        .with_state(app_state)
        // Merge module routers
        .merge(auth_routes(auth_state.clone()))
        .merge(video_routes().with_state(video_state))
        .merge(image_routes().with_state(image_state))
        // Serve static files from storage directory
        .nest_service("/storage", ServeDir::new(&storage_dir))
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

    println!("📊 SERVER ENDPOINTS:");
    println!("   • Web UI:        http://{}", addr);
    println!("   • Test Player:   http://{}/test", addr);
    println!("   • Login:         http://{}/login", addr);
    println!("   • OIDC Login:    http://{}/oidc/authorize", addr);
    println!("   • Emergency:     http://{}/login/emergency", addr);
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
    println!(
        "     -f flv \"rtmp://localhost:1935/live?token={}\"",
        RTMP_PUBLISH_TOKEN
    );

    println!("\n   Linux (Webcam + Microphone):");
    println!("   ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 \\");
    println!("     -c:v libx264 -preset veryfast -tune zerolatency \\");
    println!("     -c:a aac -b:a 128k -ar 44100 \\");
    println!(
        "     -f flv \"rtmp://localhost:1935/live?token={}\"",
        RTMP_PUBLISH_TOKEN
    );

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
    println!("   • crates/user-auth     - OIDC Authentication (Casdoor)");

    println!("\n🔐 AUTHENTICATION:");
    println!("   • Primary:   OIDC with Casdoor (Login with Appkask)");
    println!("   • Fallback:  Emergency local login");
    println!("   • Configure: Set OIDC_* environment variables");

    println!("\n{}\n", "═".repeat(64));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

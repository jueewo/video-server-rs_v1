use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{header::HeaderValue, Method, StatusCode},
    response::{Html, Json},
    routing::{delete, get, post},
    Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use time::{Duration, OffsetDateTime};
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

#[derive(Template)]
#[template(path = "demo.html")]
struct DemoTemplate {
    code: String,
    error: String,
    resources: Vec<Resource>,
}

// -------------------------------
// Access Code API Types
// -------------------------------

#[derive(Deserialize)]
struct CreateAccessCodeRequest {
    code: String,
    description: Option<String>,
    expires_at: Option<String>, // ISO 8601 datetime string
    media_items: Vec<MediaItem>,
}

#[derive(Deserialize, Serialize)]
struct MediaItem {
    media_type: String, // "video" or "image"
    media_slug: String,
}

#[derive(Serialize)]
struct AccessCodeResponse {
    id: i32,
    code: String,
    description: Option<String>,
    expires_at: Option<String>,
    created_at: String,
    media_items: Vec<MediaItem>,
}

#[derive(Serialize)]
struct AccessCodeListResponse {
    access_codes: Vec<AccessCodeResponse>,
}

#[derive(Serialize)]
struct Resource {
    media_type: String,
    slug: String,
    title: String,
}

// -------------------------------
// Access Code Management Handlers
// -------------------------------

async fn create_access_code(
    session: Session,
    State(state): State<Arc<AppState>>,
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
        .fetch_optional(&state.video_state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
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
    .fetch_one(&state.video_state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert permissions (only for owned media)
    for item in &request.media_items {
        if item.media_type != "video" && item.media_type != "image" {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate ownership
        let is_owner = match item.media_type.as_str() {
            "video" => {
                let owner: Option<String> =
                    sqlx::query_scalar("SELECT user_id FROM videos WHERE slug = ?")
                        .bind(&item.media_slug)
                        .fetch_optional(&state.video_state.pool)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                println!(
                    "🔍 Ownership check: video '{}' owner={:?}, user='{}', is_owner={}",
                    item.media_slug,
                    owner,
                    user_id,
                    owner.as_ref() == Some(&user_id)
                );
                owner.as_ref() == Some(&user_id)
            }
            "image" => {
                let owner: Option<String> =
                    sqlx::query_scalar("SELECT user_id FROM images WHERE slug = ?")
                        .bind(&item.media_slug)
                        .fetch_optional(&state.video_state.pool)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                println!(
                    "🔍 Ownership check: image '{}' owner={:?}, user='{}', is_owner={}",
                    item.media_slug,
                    owner,
                    user_id,
                    owner.as_ref() == Some(&user_id)
                );
                owner.as_ref() == Some(&user_id)
            }
            _ => false,
        };

        if !is_owner {
            println!(
                "❌ Ownership validation failed for {}/{}",
                item.media_type, item.media_slug
            );
            return Err(StatusCode::FORBIDDEN); // User doesn't own this media
        }

        sqlx::query(
            "INSERT INTO access_code_permissions (access_code_id, media_type, media_slug) VALUES (?, ?, ?)"
        )
        .bind(code_id)
        .bind(&item.media_type)
        .bind(&item.media_slug)
        .execute(&state.video_state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return created access code
    Ok(Json(AccessCodeResponse {
        id: code_id,
        code: request.code,
        description: request.description,
        expires_at: request.expires_at,
        created_at: OffsetDateTime::now_utc().to_string(),
        media_items: request.media_items,
    }))
}

async fn list_access_codes(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Json<AccessCodeListResponse>, StatusCode> {
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
    .fetch_all(&state.video_state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut access_codes = Vec::new();

    for (id, code, description, expires_at, created_at) in codes {
        // Get permissions for this code
        let permissions = sqlx::query_as::<_, (String, String)>(
            "SELECT media_type, media_slug FROM access_code_permissions WHERE access_code_id = ?",
        )
        .bind(id)
        .fetch_all(&state.video_state.pool)
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

    Ok(Json(AccessCodeListResponse { access_codes }))
}

async fn delete_access_code(
    Path(code): Path<String>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
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

    // Delete access code (only if owned by current user)
    let rows_affected = sqlx::query("DELETE FROM access_codes WHERE code = ? AND created_by = ?")
        .bind(&code)
        .bind(&user_id)
        .execute(&state.video_state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .rows_affected();

    if rows_affected == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
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

async fn demo_handler(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let code = params.get("code").cloned();
    let mut error = String::new();
    let mut resources = Vec::new();

    if let Some(ref access_code) = code {
        // Check if access code exists and not expired
        let now = OffsetDateTime::now_utc();
        let code_info: Option<(i32, Option<String>)> =
            sqlx::query_as("SELECT id, expires_at FROM access_codes WHERE code = ?")
                .bind(access_code)
                .fetch_optional(&state.video_state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some((code_id, expires_at)) = code_info {
            if let Some(exp) = expires_at {
                let exp_dt = OffsetDateTime::parse(
                    &exp,
                    &time::format_description::well_known::Iso8601::DEFAULT,
                )
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                if exp_dt < now {
                    error = "Access code has expired".to_string();
                }
            }

            if error.is_empty() {
                // Get permissions
                let permissions: Vec<(String, String)> = sqlx::query_as(
                    "SELECT media_type, media_slug FROM access_code_permissions WHERE access_code_id = ?",
                )
                .bind(code_id)
                .fetch_all(&state.video_state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                for (media_type, slug) in permissions {
                    let title = match media_type.as_str() {
                        "video" => sqlx::query_scalar("SELECT title FROM videos WHERE slug = ?")
                            .bind(&slug)
                            .fetch_optional(&state.video_state.pool)
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                            .unwrap_or_else(|| "Unknown Video".to_string()),
                        "image" => sqlx::query_scalar("SELECT title FROM images WHERE slug = ?")
                            .bind(&slug)
                            .fetch_optional(&state.video_state.pool)
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                            .unwrap_or_else(|| "Unknown Image".to_string()),
                        _ => "Unknown".to_string(),
                    };
                    resources.push(Resource {
                        media_type,
                        slug,
                        title,
                    });
                }
            }
        } else {
            error = "Invalid access code".to_string();
        }
    }

    let template = DemoTemplate {
        code: code.unwrap_or_default(),
        error,
        resources,
    };
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

    // Run migrations (skip if already applied or modified)
    // if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
    //     println!("⚠️  Migration warning: {}", e);
    //     println!("   Continuing with existing database schema...");
    // }

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

    let auth_state = match AuthState::new(oidc_config.clone(), pool.clone()).await {
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
            Arc::new(AuthState::new_without_oidc(oidc_config, pool.clone()))
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
        .route("/demo", get(demo_handler))
        .route("/health", get(health_check))
        // Access code management (authenticated)
        .route("/api/access-codes", post(create_access_code))
        .route("/api/access-codes", get(list_access_codes))
        .route("/api/access-codes/:code", delete(delete_access_code))
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
                        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
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
    println!("   ✅ access-codes     (Shared media access)");

    println!("📊 SERVER ENDPOINTS:");
    println!("   • Web UI:        http://{}", addr);
    println!("   • Demo:          http://{}/demo", addr);
    println!("   • Test Player:   http://{}/test", addr);
    println!("   • Login:         http://{}/login", addr);
    println!("   • OIDC Login:    http://{}/oidc/authorize", addr);
    println!("   • Emergency:     http://{}/login/emergency", addr);
    println!("   • Images:        http://{}/images", addr);
    println!("   • Upload:        http://{}/upload", addr);
    println!("   • Health:        http://{}/health", addr);
    println!("   • MediaMTX API:  http://{}/api/mediamtx/status", addr);
    println!("   • Access Codes:  http://{}/api/access-codes", addr);

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

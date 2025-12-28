// fn main() {
//     println!("Hello, world!");
// }

use axum::{
    extract::{Path, State},
    http::{HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use time::Duration;
use tower::util::ServiceExt;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeFile;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

// Shared app state
#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    storage_dir: PathBuf,
}

// Simple login handler (in real app: proper password check)
async fn login_handler(mut session: Session) -> Result<&'static str, StatusCode> {
    // Fake login - in production: verify username/password
    session.insert("user_id", 1u32).await.unwrap();
    session.insert("authenticated", true).await.unwrap();
    Ok("Logged in")
}

// Logout
async fn logout_handler(session: Session) -> Result<&'static str, StatusCode> {
    session.remove::<bool>("authenticated").await;
    session.remove::<u32>("user_id").await;
    Ok("Logged out")
}

// Index page listing public videos
async fn index_handler(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let videos: Vec<(String, String)> =
        sqlx::query_as("SELECT slug, title FROM videos WHERE is_public = 1")
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut html = String::from(
        "<html><head><title>Public Videos</title></head><body><h1>Public Videos</h1><ul>",
    );
    for (slug, title) in videos {
        html.push_str(&format!(
            "<li><a href=\"/hls/{}/master.m3u8\">{}</a></li>",
            slug, title
        ));
    }
    html.push_str("</ul></body></html>");
    Ok(Html(html))
}

// HLS file handler with auth check
async fn hls_handler(
    Path((slug, path)): Path<(String, String)>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    println!("HLS request: slug={}, path={}", slug, path);

    // Lookup video
    let video: Option<(bool,)> = sqlx::query_as("SELECT is_public FROM videos WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            println!("DB query error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let is_public = video.map(|(p,)| p).unwrap_or(false);
    println!("Video is_public: {}", is_public);

    // Auth check for private videos
    if !is_public {
        let authenticated: bool = session
            .get::<bool>("authenticated")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        if !authenticated {
            println!("Unauthorized access to private video");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Build file path: storage/public or storage/private
    let folder = if is_public { "public" } else { "private" };
    let file_path = state.storage_dir.join(folder).join(&slug).join(&path);
    println!("File path: {:?}", file_path);
    println!("File exists: {}", file_path.exists());

    // Serve the file (master.m3u8 or .ts segments)
    ServeFile::new(file_path)
        .oneshot(axum::http::Request::new(axum::body::Body::empty()))
        .await
        .map(|res| res.into_response())
        .map_err(|e| {
            println!("ServeFile error: {}", e);
            StatusCode::NOT_FOUND
        })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // DB setup
    println!("Connecting to DB...");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:video.db?mode=rwc")
        .await?;
    println!("DB connected");

    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?; // Create tables if needed
                                                      // Run schema.sql manually first time or add as migration
    println!("Migrations completed");

    let storage_dir = std::env::current_dir().unwrap().join("storage");
    let state = Arc::new(AppState { pool, storage_dir });

    // Session store (SQLite for persistence; use MemoryStore for dev)
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // true in prod with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/login", get(login_handler))
        .route("/logout", get(logout_handler))
        .route("/hls/:slug/*path", get(hls_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin("http://localhost:4321".parse::<HeaderValue>().unwrap())
                        // .allow_origin("https://app.appkask.com".parse::<HeaderValue>().unwrap())
                        .allow_origin(tower_http::cors::AllowOrigin::predicate(
                                |origin: &HeaderValue, _request_parts: &_| {
                                    origin.as_bytes().ends_with(b".appkask.com") ||
                                    origin.as_bytes() == b"https://appkask.com"
                                }
                            ))
                        .allow_credentials(true)
                        .allow_methods([Method::GET, Method::OPTIONS])
                        .allow_headers([
                            axum::http::header::CONTENT_TYPE,
                            axum::http::header::RANGE,
                        ]),
                )
                .layer(session_layer),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

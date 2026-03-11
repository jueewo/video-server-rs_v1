use std::net::SocketAddr;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webdav::WebdavState;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webdav=debug,tower=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:media.db".to_string());

    let storage_dir = std::env::var("STORAGE_DIR")
        .unwrap_or_else(|_| "./storage".to_string());

    let port: u16 = std::env::var("WEBDAV_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("WEBDAV_PORT must be a number");

    tracing::info!("Connecting to database: {}", database_url);
    tracing::info!("Storage dir: {}", storage_dir);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = WebdavState::new(pool, storage_dir);

    let app = webdav::webdav_routes(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("WebDAV server listening on http://{}", addr);
    tracing::info!("Auth: Basic Auth — username ignored, password must be a valid API key");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

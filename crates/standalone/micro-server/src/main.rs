//! Micro Server — minimal axum sidecar with two demo APIs.
//!
//! 1. **File Inspector** (uses `media-core` crate) — upload a file, get back
//!    its detected MIME type, media classification, and metadata.
//!
//! 2. **Notes API** (SQLite) — simple CRUD for notes, persisted to `data.db`.
//!
//! Satisfies the sidecar contract: `--port=NNNN` + `GET /health`.

use axum::extract::{Multipart, Path, State};
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port = parse_port();

    // Open (or create) a SQLite database in the working directory
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:data.db?mode=rwc")
        .await?;

    // Create the notes table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&db)
    .await?;

    let state = Arc::new(AppState { db });
    let app = router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    info!("Micro server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn parse_port() -> u16 {
    std::env::args()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.strip_prefix("--port=").unwrap().parse().ok())
        .unwrap_or(3001)
}

fn router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/info", get(info_handler))
        // media-core: file inspection
        .route("/api/inspect", post(inspect_file))
        // SQLite: notes CRUD
        .route("/api/notes", get(list_notes).post(create_note))
        .route("/api/notes/{id}", delete(delete_note))
        .with_state(state)
        .layer(cors)
}

// ── Health & Info ────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

async fn info_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "micro-server",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "apis": ["inspect", "notes"],
    }))
}

// ── File Inspector (media-core) ──────────────────────────────────

async fn inspect_file(mut multipart: Multipart) -> impl IntoResponse {
    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        _ => return Json(serde_json::json!({"error": "No file uploaded"})),
    };

    let filename = field.file_name().unwrap_or("unknown").to_string();
    let data = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return Json(serde_json::json!({"error": format!("Read failed: {e}")})),
    };

    // Use media-core for detection
    let mime_type = media_core::detect_mime_type(&data, &filename);
    let slug = media_core::generate_slug(&filename);

    let mut result = serde_json::json!({
        "filename": filename,
        "slug": slug,
        "size": data.len(),
        "mime_type": mime_type,
    });

    if let Ok(metadata) = media_core::extract_metadata(&data, filename, mime_type) {
        result["media_type"] = serde_json::json!(format!("{:?}", metadata.media_type));
        result["extension"] = serde_json::json!(metadata.extension);
    }

    Json(result)
}

// ── Notes API (SQLite) ───────────────────────────────────────────

#[derive(Serialize)]
struct Note {
    id: i64,
    title: String,
    body: String,
    created_at: String,
}

#[derive(Deserialize)]
struct CreateNote {
    title: String,
    #[serde(default)]
    body: String,
}

async fn list_notes(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows = sqlx::query("SELECT id, title, body, created_at FROM notes ORDER BY id DESC")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let notes: Vec<Note> = rows
        .iter()
        .map(|r| Note {
            id: r.get("id"),
            title: r.get("title"),
            body: r.get("body"),
            created_at: r.get("created_at"),
        })
        .collect();

    Json(notes)
}

async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateNote>,
) -> impl IntoResponse {
    let result = sqlx::query("INSERT INTO notes (title, body) VALUES (?, ?)")
        .bind(&input.title)
        .bind(&input.body)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) => Json(serde_json::json!({"ok": true, "id": r.last_insert_rowid()})),
        Err(e) => Json(serde_json::json!({"error": format!("{e}")})),
    }
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) => Json(serde_json::json!({"ok": true, "deleted": r.rows_affected()})),
        Err(e) => Json(serde_json::json!({"error": format!("{e}")})),
    }
}

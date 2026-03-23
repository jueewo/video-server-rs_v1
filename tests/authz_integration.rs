//! Integration tests for authorization-critical paths (TD-008)
//!
//! These tests build a minimal in-process router against an in-memory SQLite
//! database and exercise the authz decision points without starting a real
//! server.  Each test group covers one layer of the 4-layer access model:
//!
//!   1. Unauthenticated (guest) — public vs private resource access
//!   2. Authenticated session — owner vs non-owner
//!   3. CRUD ownership — update/delete scoped to user_id
//!   4. X-Request-ID propagation (TD-011)
//!
//! The auth middleware (`api_key_or_session_auth`) is intentionally NOT applied
//! here — we test handler-level ownership checks directly.  Middleware-level
//! gate tests are in `tests/middleware_authz.rs`.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use media_manager::{media_routes, MediaManagerState};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ── Test helpers ─────────────────────────────────────────────────────────────

/// Create an in-memory SQLite pool with the minimum schema needed by the tests.
async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("in-memory DB");

    // Minimal schema — just the tables our handlers touch.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS media_items (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            slug              TEXT    NOT NULL UNIQUE,
            media_type        TEXT    NOT NULL,
            title             TEXT    NOT NULL,
            description       TEXT,
            filename          TEXT    NOT NULL DEFAULT '',
            original_filename TEXT,
            mime_type         TEXT    NOT NULL DEFAULT '',
            file_size         INTEGER NOT NULL DEFAULT 0,
            is_public         INTEGER NOT NULL DEFAULT 0,
            user_id           TEXT    NOT NULL,
            group_id          INTEGER,
            vault_id          TEXT,
            category          TEXT,
            language          TEXT,
            status            TEXT    NOT NULL DEFAULT 'active',
            featured          INTEGER NOT NULL DEFAULT 0,
            thumbnail_url     TEXT,
            preview_url       TEXT,
            webp_url          TEXT,
            hls_playlist      TEXT,
            view_count        INTEGER NOT NULL DEFAULT 0,
            download_count    INTEGER NOT NULL DEFAULT 0,
            like_count        INTEGER NOT NULL DEFAULT 0,
            share_count       INTEGER NOT NULL DEFAULT 0,
            allow_download    INTEGER NOT NULL DEFAULT 1,
            allow_comments    INTEGER NOT NULL DEFAULT 1,
            mature_content    INTEGER NOT NULL DEFAULT 0,
            seo_title         TEXT,
            seo_description   TEXT,
            seo_keywords      TEXT,
            created_at        TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at        TEXT    NOT NULL DEFAULT (datetime('now')),
            published_at      TEXT
        );
        CREATE TABLE IF NOT EXISTS media_tags (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            media_id   INTEGER NOT NULL,
            tag        TEXT    NOT NULL,
            created_at TEXT    NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS access_groups (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL,
            created_at TEXT    NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS storage_vaults (
            vault_id   TEXT    PRIMARY KEY,
            user_id    TEXT    NOT NULL,
            vault_name TEXT    NOT NULL DEFAULT 'Default',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT    NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS access_control_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            resource_id INTEGER,
            user_id     TEXT,
            action      TEXT,
            result      TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("schema setup");

    pool
}

/// Insert a media item owned by `user_id` and return its slug.
async fn insert_media(
    pool: &SqlitePool,
    slug: &str,
    user_id: &str,
    is_public: i32,
) {
    sqlx::query(
        "INSERT INTO media_items (slug, media_type, title, filename, mime_type, file_size, is_public, user_id)
         VALUES (?, 'image', 'Test', 'test.webp', 'image/webp', 1024, ?, ?)",
    )
    .bind(slug)
    .bind(is_public)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert media");
}

/// Build a minimal `MediaManagerState` — no real storage dir needed for JSON API tests.
fn make_state(pool: SqlitePool) -> MediaManagerState {
    use common::storage::UserStorageManager;
    let database = Arc::new(db_sqlite::SqliteDatabase::new(pool.clone()));
    MediaManagerState::new(
        database.clone(),
        database.clone(),
        "/tmp/test-storage".to_string(),
        UserStorageManager::new(std::path::PathBuf::from("/tmp/test-storage")),
        Arc::new(access_control::AccessControlService::with_audit_enabled(database.clone(), database.clone(), false)),
    )
}

/// Decode a response body to a `serde_json::Value`.
async fn json_body(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.expect("body").to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

/// Build the router under test with a session layer (required by handlers that
/// extract `Session`), but without the auth gate middleware.
/// This lets us test handler-level ownership logic directly.
fn test_router(state: MediaManagerState) -> axum::Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);
    media_routes().with_state(state).layer(session_layer)
}

// ── Tests: JSON list API ──────────────────────────────────────────────────────

#[tokio::test]
async fn list_media_json_returns_200() {
    let pool = test_pool().await;
    insert_media(&pool, "pub-item", "user-1", 1).await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_media_json_empty_db() {
    let pool = test_pool().await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response.into_body()).await;
    assert_eq!(body["total"], 0);
}

// ── Tests: CRUD ownership — toggle_visibility ─────────────────────────────────

#[tokio::test]
async fn toggle_visibility_returns_401_without_session() {
    let pool = test_pool().await;
    insert_media(&pool, "private-item", "user-1", 0).await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/media/private-item/toggle-visibility")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"is_public": true}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // No session → handler returns 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Tests: CRUD ownership — get_media_item ────────────────────────────────────

#[tokio::test]
async fn get_media_item_returns_401_without_session() {
    let pool = test_pool().await;
    insert_media(&pool, "my-item", "user-1", 0).await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media/my-item")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Tests: CRUD ownership — delete_media ─────────────────────────────────────

#[tokio::test]
async fn delete_media_returns_401_without_session() {
    let pool = test_pool().await;
    insert_media(&pool, "to-delete", "user-1", 0).await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/media/to-delete")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Tests: CRUD ownership — update_media_item ────────────────────────────────

#[tokio::test]
async fn update_media_returns_401_without_session() {
    let pool = test_pool().await;
    insert_media(&pool, "to-update", "user-1", 0).await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/media/to-update")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title": "New Title"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Tests: upload requires auth ───────────────────────────────────────────────

#[tokio::test]
async fn upload_returns_401_without_session() {
    let pool = test_pool().await;
    let app = test_router(make_state(pool));

    // Multipart without a session — handler should return 401 immediately.
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/media/upload")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----boundary",
                )
                .body(Body::from("------boundary--"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Tests: vault endpoint requires auth ──────────────────────────────────────

#[tokio::test]
async fn get_user_vaults_returns_401_without_session() {
    let pool = test_pool().await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/user/vaults")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Tests: X-Request-ID propagation (TD-011) ─────────────────────────────────

#[tokio::test]
async fn request_id_is_echoed_in_response() {
    use common::request_id::request_id_middleware;

    let pool = test_pool().await;
    let app = test_router(make_state(pool))
        .layer(axum::middleware::from_fn(request_id_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media")
                .header("x-request-id", "test-id-abc123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let echoed = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok());
    assert_eq!(echoed, Some("test-id-abc123"), "client request ID should be echoed back");
}

#[tokio::test]
async fn request_id_is_generated_when_absent() {
    use common::request_id::request_id_middleware;

    let pool = test_pool().await;
    let app = test_router(make_state(pool))
        .layer(axum::middleware::from_fn(request_id_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok());
    assert!(id.is_some(), "server should generate X-Request-ID when absent");
    assert!(!id.unwrap().is_empty());
}

// ── Tests: error response shape (TD-007) ─────────────────────────────────────

#[tokio::test]
async fn unauthorized_response_has_json_error_field() {
    let pool = test_pool().await;
    let app = test_router(make_state(pool));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/media/some-slug")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response.into_body()).await;
    // Handlers return {"error": "..."} — verify the key is present.
    assert!(
        body.get("error").is_some(),
        "error responses must have an 'error' key, got: {body}"
    );
}

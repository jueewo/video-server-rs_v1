//! Optional DB persistence for client-side sql.js (WASM) apps.
//!
//! When `meta.yaml` contains `db_writable: true`, the platform exposes:
//! - `POST /api/app-db/{workspace_id}/{*folder_path}` — save the DB blob
//! - `GET  /api/app-db/{workspace_id}/{*folder_path}` — check if writable
//!
//! The frontend exports the sql.js database as a Uint8Array and POSTs it here.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::AppRuntimeState;

#[derive(Deserialize, Default)]
struct AppMeta {
    #[serde(default)]
    db_writable: bool,
}

/// Check if the app has db_writable enabled and return the app directory.
fn resolve_writable_app(
    storage_base: &PathBuf,
    workspace_id: &str,
    folder_path: &str,
) -> Result<PathBuf, Response> {
    let app_dir = storage_base
        .join("workspaces")
        .join(workspace_id)
        .join(folder_path);

    if !app_dir.exists() {
        return Err((StatusCode::NOT_FOUND, "App folder not found").into_response());
    }

    // Read meta.yaml and check db_writable
    let meta_path = app_dir.join("meta.yaml");
    let meta: AppMeta = if meta_path.exists() {
        let content = std::fs::read_to_string(&meta_path).unwrap_or_default();
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        AppMeta::default()
    };

    if !meta.db_writable {
        return Err((
            StatusCode::FORBIDDEN,
            "Database writes not enabled. Set db_writable: true in meta.yaml",
        )
            .into_response());
    }

    Ok(app_dir)
}

/// GET /api/app-db/{workspace_id}/{*folder_path}
/// Returns whether the app's DB is writable (for frontend feature detection).
pub async fn db_status_handler(
    State(state): State<Arc<AppRuntimeState>>,
    Path((workspace_id, folder_path)): Path<(String, String)>,
) -> Response {
    match resolve_writable_app(&state.storage_base, &workspace_id, &folder_path) {
        Ok(app_dir) => {
            let db_path = app_dir.join("data.db");
            let exists = db_path.exists();
            let size = if exists {
                std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            axum::Json(serde_json::json!({
                "writable": true,
                "exists": exists,
                "size": size,
            }))
            .into_response()
        }
        Err(resp) => resp,
    }
}

/// POST /api/app-db/{workspace_id}/{*folder_path}
/// Accepts raw bytes (the exported sql.js database) and writes to data.db.
pub async fn db_save_handler(
    State(state): State<Arc<AppRuntimeState>>,
    Path((workspace_id, folder_path)): Path<(String, String)>,
    body: Body,
) -> Response {
    let app_dir = match resolve_writable_app(&state.storage_base, &workspace_id, &folder_path) {
        Ok(dir) => dir,
        Err(resp) => return resp,
    };

    // Read the body (limit: 50 MB for database files)
    let bytes = match axum::body::to_bytes(body, 50 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to read DB upload body: {}", e);
            return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
        }
    };

    if bytes.is_empty() {
        return (StatusCode::BAD_REQUEST, "Empty body").into_response();
    }

    // Basic SQLite validation: check magic header
    if bytes.len() < 16 || &bytes[..16] != b"SQLite format 3\0" {
        return (StatusCode::BAD_REQUEST, "Not a valid SQLite database").into_response();
    }

    let db_path = app_dir.join("data.db");

    // Write atomically: write to temp file, then rename
    let tmp_path = app_dir.join("data.db.tmp");
    if let Err(e) = tokio::fs::write(&tmp_path, &bytes).await {
        error!("Failed to write temp DB file: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save database").into_response();
    }
    if let Err(e) = tokio::fs::rename(&tmp_path, &db_path).await {
        error!("Failed to rename temp DB file: {}", e);
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save database").into_response();
    }

    let size = bytes.len();
    info!(
        "Saved DB for {}/{}: {} bytes",
        workspace_id, folder_path, size
    );

    axum::Json(serde_json::json!({
        "ok": true,
        "size": size,
    }))
    .into_response()
}

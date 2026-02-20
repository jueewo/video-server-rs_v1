//! 3D Gallery - Immersive Media Viewing
//!
//! This crate provides a 3D virtual gallery for viewing images and videos
//! in an immersive WebGL environment using Babylon.js.
//!
//! Access is via access codes (no authentication required).

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;

pub mod api;
pub mod models;
pub mod routes;

/// Serve JavaScript files with correct MIME type
/// Handles index.js, bundle.js, and all chunk files
async fn serve_js_file(filename: &str) -> Response {
    // Use GALLERY_STATIC_DIR environment variable, or fall back to common paths
    let base_dir = std::env::var("GALLERY_STATIC_DIR").unwrap_or_else(|_| ".".to_string());

    let paths = [
        format!("{}/crates/3d-gallery/static/{}", base_dir, filename),
        format!("{}/static/3d-gallery/{}", base_dir, filename),
        format!("crates/3d-gallery/static/{}", filename),
        format!("static/3d-gallery/{}", filename),
    ];

    for path in &paths {
        if let Ok(content) = tokio::fs::read(path).await {
            tracing::info!("✅ Served {} from: {}", filename, path);

            // Determine MIME type based on file extension
            let mime_type = if filename.ends_with(".js") {
                "application/javascript; charset=utf-8"
            } else if filename.ends_with(".js.map") {
                "application/json; charset=utf-8"
            } else {
                "application/octet-stream"
            };

            return (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], content).into_response();
        }
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    tracing::error!(
        "🚫 {} not found. CWD: {:?}, tried paths: {:?}",
        filename,
        cwd,
        paths
    );
    (
        StatusCode::NOT_FOUND,
        format!("File not found: {}", filename),
    )
        .into_response()
}

/// Handler for bundle.js (backward compatibility)
async fn serve_bundle_js() -> Response {
    serve_js_file("bundle.js").await
}

/// Handler for bundle.js.map (backward compatibility)
async fn serve_bundle_js_map() -> Response {
    serve_js_file("bundle.js.map").await
}

/// Handler for index.js (new code-split entry point)
async fn serve_index_js() -> Response {
    serve_js_file("index.js").await
}

/// Handler for index.js.map
async fn serve_index_js_map() -> Response {
    serve_js_file("index.js.map").await
}

/// Handler for chunk files (e.g., chunk-ABC123.js)
async fn serve_chunk_js(axum::extract::Path(filename): axum::extract::Path<String>) -> Response {
    // Validate filename to prevent directory traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        tracing::warn!("🚫 Invalid filename attempted: {}", filename);
        return (StatusCode::BAD_REQUEST, "Invalid filename").into_response();
    }

    serve_js_file(&filename).await
}

/// Create the 3D gallery router with database pool
///
/// Routes:
/// - GET /3d - Main 3D viewer page (requires ?code= parameter)
/// - GET /digital-twin - Alternative route to viewer
/// - GET /api/3d/gallery - JSON API for gallery data
/// - GET /static/3d-gallery/* - Static assets (index.js, bundle.js, chunks, etc.)
pub fn router(pool: Arc<SqlitePool>) -> Router {
    Router::new()
        // Main viewer pages
        .route("/3d", get(routes::viewer_page))
        .route("/digital-twin", get(routes::viewer_page))
        // API endpoints (with database state)
        .route("/api/3d/gallery", get(api::get_gallery_data))
        .with_state(pool)
        // Static file serving for frontend with proper MIME types
        // New code-split entry point
        .route("/static/3d-gallery/index.js", get(serve_index_js))
        .route("/static/3d-gallery/index.js.map", get(serve_index_js_map))
        // Legacy bundle.js (backward compatibility)
        .route("/static/3d-gallery/bundle.js", get(serve_bundle_js))
        .route("/static/3d-gallery/bundle.js.map", get(serve_bundle_js_map))
        // Chunk files (e.g., chunk-ABC123.js, chunk-ABC123.js.map)
        .route("/static/3d-gallery/{filename}", get(serve_chunk_js))
}

// Tests removed - router requires database pool parameter

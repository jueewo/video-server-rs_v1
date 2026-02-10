//! 3D Gallery - Immersive Media Viewing
//!
//! This crate provides a 3D virtual gallery for viewing images and videos
//! in an immersive WebGL environment using Babylon.js.
//!
//! Access is via access codes (no authentication required).

use axum::{
    body::Bytes,
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

/// Serve bundle.js with correct MIME type
async fn serve_bundle_js() -> Response {
    // Use GALLERY_STATIC_DIR environment variable, or fall back to common paths
    let base_dir = std::env::var("GALLERY_STATIC_DIR")
        .unwrap_or_else(|_| ".".to_string());

    let paths = [
        format!("{}/crates/3d-gallery/static/bundle.js", base_dir),
        format!("{}/static/3d-gallery/bundle.js", base_dir),
        "crates/3d-gallery/static/bundle.js".to_string(),
        "static/3d-gallery/bundle.js".to_string(),
    ];

    for path in &paths {
        if let Ok(content) = tokio::fs::read(path).await {
            tracing::info!("✅ Served bundle.js from: {}", path);
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
                content,
            )
                .into_response();
        }
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    tracing::error!("🚫 bundle.js not found. CWD: {:?}, tried paths: {:?}", cwd, paths);
    (StatusCode::NOT_FOUND, format!("File not found. CWD: {:?}", cwd)).into_response()
}

/// Serve bundle.js.map with correct MIME type
async fn serve_bundle_js_map() -> Response {
    // Use GALLERY_STATIC_DIR environment variable, or fall back to common paths
    let base_dir = std::env::var("GALLERY_STATIC_DIR")
        .unwrap_or_else(|_| ".".to_string());

    let paths = [
        format!("{}/crates/3d-gallery/static/bundle.js.map", base_dir),
        format!("{}/static/3d-gallery/bundle.js.map", base_dir),
        "crates/3d-gallery/static/bundle.js.map".to_string(),
        "static/3d-gallery/bundle.js.map".to_string(),
    ];

    for path in &paths {
        if let Ok(content) = tokio::fs::read(path).await {
            tracing::info!("✅ Served bundle.js.map from: {}", path);
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
                content,
            )
                .into_response();
        }
    }

    tracing::error!("🚫 bundle.js.map not found");
    (StatusCode::NOT_FOUND, "File not found").into_response()
}

/// Create the 3D gallery router with database pool
///
/// Routes:
/// - GET /3d - Main 3D viewer page (requires ?code= parameter)
/// - GET /digital-twin - Alternative route to viewer
/// - GET /api/3d/gallery - JSON API for gallery data
/// - GET /static/3d-gallery/* - Static assets (bundle.js, etc.)
pub fn router(pool: Arc<SqlitePool>) -> Router {
    Router::new()
        // Main viewer pages
        .route("/3d", get(routes::viewer_page))
        .route("/digital-twin", get(routes::viewer_page))
        // API endpoints (with database state)
        .route("/api/3d/gallery", get(api::get_gallery_data))
        .with_state(pool)
        // Static file serving for frontend bundle with proper MIME types
        .route("/static/3d-gallery/bundle.js", get(serve_bundle_js))
        .route("/static/3d-gallery/bundle.js.map", get(serve_bundle_js_map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router = router();
        // If this compiles, router is valid
    }
}

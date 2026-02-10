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
    match tokio::fs::read("crates/3d-gallery/static/bundle.js").await {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
            content,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

/// Serve bundle.js.map with correct MIME type
async fn serve_bundle_js_map() -> Response {
    match tokio::fs::read("crates/3d-gallery/static/bundle.js.map").await {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
            content,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
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

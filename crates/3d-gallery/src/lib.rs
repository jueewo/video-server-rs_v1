//! 3D Gallery - Immersive Media Viewing
//!
//! This crate provides a 3D virtual gallery for viewing images and videos
//! in an immersive WebGL environment using Babylon.js.
//!
//! Access is via access codes (no authentication required).

use axum::{routing::get, Router};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub mod api;
pub mod models;
pub mod routes;

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
        // Static file serving for frontend bundle
        .nest_service(
            "/static/3d-gallery",
            ServeDir::new("crates/3d-gallery/static"),
        )
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

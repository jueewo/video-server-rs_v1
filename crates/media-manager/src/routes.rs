//! Routes for unified media management

use axum::{
    routing::{get, post},
    Router,
};
use common::storage::UserStorageManager;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Media manager state
#[derive(Clone)]
pub struct MediaManagerState {
    pub pool: SqlitePool,
    pub storage_dir: String,
    pub user_storage: UserStorageManager,
    pub access_control: Arc<access_control::AccessControlService>,
}

impl MediaManagerState {
    pub fn new(
        pool: SqlitePool,
        storage_dir: String,
        user_storage: UserStorageManager,
        access_control: Arc<access_control::AccessControlService>,
    ) -> Self {
        Self {
            pool,
            storage_dir,
            user_storage,
            access_control,
        }
    }
}

/// Create media routes
pub fn media_routes() -> Router<MediaManagerState> {
    Router::new()
        // Upload - temporarily commented out, using media-hub's upload for now
        // .route("/api/media/upload", post(crate::upload::upload_media))

        // Detail page (HTML) - /media handled by media-hub
        .route("/media/:slug", get(crate::detail::media_detail_handler))

        // List & Detail (API) - also commented to avoid conflict with media-hub
        // .route("/api/media", get(list_media))
        // .route("/api/media/:slug", get(get_media_detail))

        // Image serving
        .route("/images/:slug", get(crate::serve::serve_image_with_suffix_check))
        .route("/images/:slug/original", get(crate::serve::serve_image_original))
        .route("/images/:slug/thumb", get(crate::serve::serve_image_thumbnail))
}

/// List media items (API)
async fn list_media() -> &'static str {
    "Media list endpoint - TODO"
}

/// Get media detail (API)
async fn get_media_detail() -> &'static str {
    "Media detail endpoint - TODO"
}

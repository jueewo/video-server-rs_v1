//! Routes for unified media management
//!
//! Consolidates all media endpoints:
//! - Listing and search (HTML + JSON API)
//! - Upload form and upload handler
//! - Detail pages (HTML)
//! - Markdown view/edit/save
//! - Image serving (original, WebP, thumbnail)
//! - CRUD operations (get, update, delete, toggle visibility)
//! - Vault management

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use common::storage::UserStorageManager;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Media manager state — single shared state for all media operations
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

/// Create all media routes (unified)
///
/// This replaces both the old media-hub routes and the old media-manager routes
/// in a single router with no overlapping endpoints.
pub fn media_routes() -> Router<MediaManagerState> {
    Router::new()
        // ── Listing & Search (HTML) ─────────────────────────────────
        .route("/media", get(crate::list::list_media_html))
        .route("/media/search", get(crate::list::search_media_html))
        // ── Listing & Search (JSON API) ─────────────────────────────
        .route("/api/media", get(crate::list::list_media_json))
        .route("/api/media/search", get(crate::list::search_media_json))
        // ── Upload ──────────────────────────────────────────────────
        .route("/media/upload", get(crate::list::show_upload_form))
        .route("/api/media/upload", post(crate::upload::upload_media))
        // ── Vault management ────────────────────────────────────────
        .route("/api/user/vaults", get(crate::list::get_user_vaults))
        // ── Detail pages (HTML) ─────────────────────────────────────
        .route("/media/:slug", get(crate::detail::media_detail_handler))
        .route(
            "/media/:slug/view",
            get(crate::markdown_view::view_markdown_handler),
        )
        .route(
            "/media/:slug/edit",
            get(crate::markdown_view::edit_markdown_handler),
        )
        // ── Media CRUD (JSON API) ───────────────────────────────────
        .route(
            "/api/media/:slug/toggle-visibility",
            post(crate::list::toggle_visibility),
        )
        .route(
            "/api/media/:slug/save",
            post(crate::markdown_view::save_markdown_handler),
        )
        .route(
            "/api/media/:slug",
            get(crate::list::get_media_item)
                .put(crate::list::update_media_item)
                .delete(crate::list::delete_media),
        )
        // ── Image serving ───────────────────────────────────────────
        .route(
            "/images/:slug",
            get(crate::serve::serve_image_with_suffix_check),
        )
        .route(
            "/images/:slug/original",
            get(crate::serve::serve_image_original),
        )
        .route(
            "/images/:slug/thumb",
            get(crate::serve::serve_image_thumbnail),
        )
}

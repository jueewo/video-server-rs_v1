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
    routing::{get, post},
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
        // ── Vault management ────────────────────────────────────────
        .route("/api/user/vaults", get(crate::list::get_user_vaults))
        // ── Group selector ──────────────────────────────────────────
        .route("/api/media/groups", get(crate::list::list_user_groups))
        // ── Detail pages (HTML) ─────────────────────────────────────
        .route("/media/{slug}", get(crate::detail::media_detail_handler))
        .route(
            "/media/{slug}/view",
            get(crate::markdown_view::view_markdown_handler),
        )
        .route(
            "/media/{slug}/edit",
            get(crate::markdown_view::edit_markdown_handler),
        )
        .route(
            "/media/{slug}/bpmn",
            get(crate::bpmn_view::view_bpmn_handler),
        )
        .route("/media/{slug}/pdf", get(crate::pdf_view::view_pdf_handler))
        // ── Media CRUD (JSON API) ───────────────────────────────────
        .route(
            "/api/media/{slug}/toggle-visibility",
            post(crate::list::toggle_visibility),
        )
        .route(
            "/api/media/{slug}/save",
            post(crate::markdown_view::save_markdown_handler),
        )
        .route(
            "/api/media/{slug}/save-bpmn",
            post(crate::bpmn_view::save_bpmn_handler),
        )
        .route(
            "/api/media/{slug}",
            get(crate::list::get_media_item)
                .put(crate::list::update_media_item)
                .delete(crate::list::delete_media),
        )
}

/// Create media upload routes (strict rate limiting)
///
/// These routes handle resource-intensive upload operations and should have
/// moderate rate limits (15 RPM) to prevent abuse and resource exhaustion.
pub fn media_upload_routes() -> Router<MediaManagerState> {
    Router::new()
        // ── Upload ──────────────────────────────────────────────────
        .route("/media/upload", get(crate::list::show_upload_form))
        .route("/api/media/upload", post(crate::upload::upload_media))
}

/// Create media serving routes (lenient rate limiting)
///
/// These routes serve media files (images, PDFs, videos) and need high rate limits
/// (300+ RPM) to support galleries loading many assets concurrently. Access is
/// controlled by access codes, not rate limiting.
pub fn media_serving_routes() -> Router<MediaManagerState> {
    Router::new()
        // ── PDF serving ─────────────────────────────────────────────
        .route(
            "/media/{slug}/serve",
            get(crate::pdf_view::serve_pdf_handler),
        )
        // ── Document thumbnails ─────────────────────────────────────
        .route(
            "/documents/{slug}/thumbnail",
            get(crate::serve::serve_document_thumbnail),
        )
        // ── Image serving ───────────────────────────────────────────
        .route(
            "/images/{slug}",
            get(crate::serve::serve_image_with_suffix_check),
        )
        .route(
            "/images/{slug}/original",
            get(crate::serve::serve_image_original),
        )
        .route(
            "/images/{slug}/thumb",
            get(crate::serve::serve_image_thumbnail),
        )
}

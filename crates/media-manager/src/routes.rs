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
    // Optional video processing components (needed for HLS transcoding)
    pub video_progress_tracker: Option<video_manager::progress::ProgressTracker>,
    pub video_metrics_store: Option<video_manager::metrics::MetricsStore>,
    pub video_audit_logger: Option<video_manager::metrics::AuditLogger>,
    // HLS transcoding progress tracker (for WebSocket updates)
    pub hls_progress: Arc<crate::progress::ProgressTracker>,
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
            video_progress_tracker: None,
            video_metrics_store: None,
            video_audit_logger: None,
            hls_progress: Arc::new(crate::progress::ProgressTracker::new()),
        }
    }

    /// Create a new state with video processing support
    pub fn with_video_processing(
        pool: SqlitePool,
        storage_dir: String,
        user_storage: UserStorageManager,
        access_control: Arc<access_control::AccessControlService>,
        progress_tracker: video_manager::progress::ProgressTracker,
        metrics_store: video_manager::metrics::MetricsStore,
        audit_logger: video_manager::metrics::AuditLogger,
    ) -> Self {
        Self {
            pool,
            storage_dir,
            user_storage,
            access_control,
            video_progress_tracker: Some(progress_tracker),
            video_metrics_store: Some(metrics_store),
            video_audit_logger: Some(audit_logger),
            hls_progress: Arc::new(crate::progress::ProgressTracker::new()),
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
        // ── Tag autocomplete (user-scoped) ──────────────────────────
        .route("/api/media/tags/search", get(crate::list::search_user_tags))
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
        // ── Video progress tracking ─────────────────────────────────
        .route(
            "/api/media/{slug}/progress",
            get(crate::upload::get_upload_progress),
        )
        .route(
            "/api/media/{slug}/progress/ws",
            get(crate::upload::progress_websocket),
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

/// Create folder access code routes (public, no auth — validated by code)
pub fn folder_access_routes() -> Router<MediaManagerState> {
    Router::new().route(
        "/api/folder/{code}/media",
        get(crate::folder_access::folder_media_by_code),
    )
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
        // ── Image serving ───────────────────────────────────────────
        .route(
            "/media/{slug}/image.webp",
            get(crate::serve::serve_image_webp),
        )
        // ── Thumbnail serving (all media types) ──────────────────────
        .route(
            "/media/{slug}/thumbnail",
            get(crate::serve::serve_thumbnail),
        )
    // NOTE: HLS video serving (/hls/{slug}/{*path}) is handled by video-manager crate
}

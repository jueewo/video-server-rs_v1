//! Legacy route redirects for backward compatibility
//!
//! These redirects ensure that old links to /images and /documents
//! continue to work by redirecting to the new unified /media routes.

use axum::{extract::Path, response::Redirect, routing::get, Router};

/// Create legacy redirect routes
pub fn legacy_redirect_routes() -> Router {
    Router::new()
        // Image routes
        .route("/images", get(redirect_images_list))
        .route("/images/view/:slug", get(redirect_image_view))
        // Document routes
        .route("/documents", get(redirect_documents_list))
        .route("/documents/:slug", get(redirect_document_view))
}

/// Redirect /images to unified media view filtered by images
async fn redirect_images_list() -> Redirect {
    Redirect::permanent("/media?type=image")
}

/// Redirect /images/view/:slug to image serving endpoint
async fn redirect_image_view(Path(slug): Path<String>) -> Redirect {
    Redirect::permanent(&format!("/images/{}", slug))
}

/// Redirect /documents to unified media view filtered by documents
async fn redirect_documents_list() -> Redirect {
    Redirect::permanent("/media?type=document")
}

/// Redirect /documents/:slug to media detail/viewer
/// For markdown files, this will auto-redirect to the viewer with preview/raw toggle
async fn redirect_document_view(Path(slug): Path<String>) -> Redirect {
    Redirect::permanent(&format!("/media/{}", slug))
}

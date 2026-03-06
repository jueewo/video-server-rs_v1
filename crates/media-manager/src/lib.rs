//! Unified Media Manager
//!
//! Handles upload, processing, serving, listing, search, and CRUD for all media types
//! (videos, images, documents).
//!
//! This crate consolidates functionality that was previously split between
//! media-hub (listing/search/CRUD UI) and media-manager (upload/detail/serving).

pub mod bpmn_view;
pub mod detail;
pub mod folder_renderer;
pub mod list;
pub mod markdown_view;
pub mod models;
pub mod pdf_thumbnail;
pub mod pdf_view;
pub mod progress;
pub mod routes;
pub mod search;
pub mod serve;
pub mod templates;
pub mod upload;

pub use folder_renderer::MediaFolderRenderer;
pub use routes::{media_routes, media_serving_routes, media_upload_routes, MediaManagerState};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

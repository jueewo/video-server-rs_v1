//! Unified Media Manager
//!
//! Handles upload, processing, and serving for all media types (videos, images, documents)
//! Replaces separate video-manager, image-manager, and document-manager crates

pub mod detail;
pub mod routes;
pub mod serve;
pub mod upload;

pub use routes::{media_routes, MediaManagerState};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

//! Unified Media Manager
//!
//! Handles upload, processing, serving, listing, search, and CRUD for all media types
//! (videos, images, documents).
//!
//! This crate consolidates functionality that was previously split between
//! media-hub (listing/search/CRUD UI) and media-manager (upload/detail/serving).

pub mod bpmn_view;
pub mod detail;
pub mod pdf_view;
pub mod list;
pub mod markdown_view;
pub mod models;
pub mod routes;
pub mod search;
pub mod serve;
pub mod templates;
pub mod upload;

pub use routes::{media_routes, MediaManagerState};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

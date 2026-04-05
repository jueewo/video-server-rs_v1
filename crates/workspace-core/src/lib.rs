//! Core traits and types for the workspace folder-type system.
//!
//! # Overview
//!
//! The workspace browser is a thin shell. Each folder type registers a
//! [`FolderTypeRenderer`] that owns the content area when that folder is opened.
//! The shell provides the outer chrome (breadcrumbs, nav, workspace name);
//! the renderer provides the inner view (media grid, diagram list, lesson outline, …).
//!
//! # Adding a new folder type
//!
//! 1. Create a crate (e.g. `crates/my-feature/`)
//! 2. Implement [`FolderTypeRenderer`] in that crate
//! 3. Add it as a dep in `crates/workspace-renderers/Cargo.toml` and call
//!    `state.register_renderer(Arc::new(MyFeatureRenderer::new(...)))` in
//!    `workspace_renderers::register_all`. `main.rs` does not need to change.
//! 4. Add a `my-feature.yaml` to `storage/folder-type-registry/`
//!
//! No changes to `workspace-manager` or `main.rs` are required.
//!
//! # Shared infrastructure
//!
//! This crate also provides:
//! - [`auth`] — shared authentication/authorization helpers for workspace routes
//! - [`WorkspaceConfig`] — workspace.yaml parsing and folder config management

pub mod auth;
mod workspace_config;

pub use workspace_config::{FolderConfig, FolderType, WorkspaceConfig};

use async_trait::async_trait;
use axum::{Router, http::StatusCode, response::Response};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Context passed to every renderer at render time
// ============================================================================

/// Request-scoped context the workspace shell passes to a [`FolderTypeRenderer`].
#[derive(Debug, Clone)]
pub struct FolderViewContext {
    pub workspace_id: String,
    pub workspace_name: String,
    pub folder_path: String,
    pub folder_name: String,
    pub user_id: String,
    pub workspace_root: PathBuf,
    pub metadata: HashMap<String, serde_yaml::Value>,
}

impl FolderViewContext {
    pub fn meta_str(&self, key: &str) -> Option<&str> {
        self.metadata.get(key)?.as_str()
    }
}

// ============================================================================
// The renderer trait
// ============================================================================

/// A folder-type renderer provides the content area for a typed workspace folder.
#[async_trait]
pub trait FolderTypeRenderer: Send + Sync {
    /// The folder type id this renderer handles, e.g. `"media-server"`.
    fn type_id(&self) -> &str;

    /// Render the content area for this folder.
    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode>;

    /// Optional additional Axum routes this renderer needs.
    fn extra_routes(&self) -> Option<Router> {
        None
    }
}

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
//! # Dual-use crates
//!
//! Each functional crate is a library usable in two modes:
//! - **Embedded**: implements [`FolderTypeRenderer`], inline view inside the workspace browser
//! - **Standalone**: exposes its own `Router` + minimal shell for independent deployment
//!
//! The standalone binary in `crates/standalone/` is a thin wrapper (~10 lines) that
//! loads config and calls into the library crate.

use async_trait::async_trait;
use axum::{Router, http::StatusCode, response::Response};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Context passed to every renderer at render time
// ============================================================================

/// Request-scoped context the workspace shell passes to a [`FolderTypeRenderer`].
///
/// The renderer holds its own database pool, storage manager, and any other
/// long-lived state as fields set at construction time. This struct carries
/// only what varies per request.
#[derive(Debug, Clone)]
pub struct FolderViewContext {
    /// The workspace this folder belongs to.
    pub workspace_id: String,

    /// Human-readable workspace name — for breadcrumbs / page titles.
    pub workspace_name: String,

    /// Full path of the folder within the workspace, e.g. `"marketing/videos"`.
    /// Empty string means the workspace root (should not happen for typed folders).
    pub folder_path: String,

    /// Just the last path segment, e.g. `"videos"`. Derived from `folder_path`
    /// for convenience — renderers should use this for display.
    pub folder_name: String,

    /// Authenticated user making the request.
    pub user_id: String,

    /// Absolute filesystem path to the workspace root directory.
    /// Use `workspace_root.join(&folder_path)` to get the folder's absolute path.
    pub workspace_root: PathBuf,

    /// Folder metadata from `workspace.yaml` — free-form key/value pairs.
    /// For `media-server` folders this includes `vault_id`.
    /// Renderers read from here; they do not write back.
    pub metadata: HashMap<String, serde_yaml::Value>,
}

impl FolderViewContext {
    /// Convenience: get a metadata string value by key.
    pub fn meta_str(&self, key: &str) -> Option<&str> {
        self.metadata.get(key)?.as_str()
    }
}

// ============================================================================
// The renderer trait
// ============================================================================

/// A folder-type renderer provides the content area for a typed workspace folder.
///
/// Implement this trait in your functional crate. The workspace browser shell
/// calls [`render_folder_view`] when the user opens a folder whose type matches
/// [`type_id`], and merges [`extra_routes`] into the router at startup.
///
/// ## Contract
///
/// - [`render_folder_view`] returns an HTML fragment or a full `Response`.
///   It will be embedded inside the workspace browser chrome — do **not** include
///   a full `<html>` page layout; the shell provides that.
/// - [`extra_routes`] are mounted at the application root. Use them for any API
///   endpoints your content view needs (e.g. `GET /api/media/list`).
/// - Implementations must be `Send + Sync` (they are stored in an `Arc`).
#[async_trait]
pub trait FolderTypeRenderer: Send + Sync {
    /// The folder type id this renderer handles, e.g. `"media-server"`.
    /// Must match the `id` field in the corresponding `*.yaml` registry file.
    fn type_id(&self) -> &str;

    /// Render the content area for this folder.
    ///
    /// Return an HTML fragment that the workspace shell will embed in its layout,
    /// or any other `Response` (e.g. a redirect or an error page).
    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode>;

    /// Optional additional Axum routes this renderer needs.
    ///
    /// Mounted at the application root during startup. Return `None` if your
    /// renderer only needs routes that are already registered elsewhere.
    fn extra_routes(&self) -> Option<Router> {
        None
    }
}

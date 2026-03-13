//! Registers all workspace folder-type renderers.
//!
//! # Adding a new renderer
//!
//! 1. Create a crate (e.g. `crates/my-feature/`) and implement [`FolderTypeRenderer`].
//! 2. Add it as a dependency in this crate's `Cargo.toml`.
//! 3. Call `state.register_renderer(Arc::new(MyFeatureRenderer::new(...)))` in
//!    [`register_all`] below.
//! 4. Add a `my-feature.yaml` to `storage/folder-type-registry/`.
//!
//! `main.rs` never needs to change.

use std::sync::Arc;

use bpmn_viewer::BpmnFolderRenderer;
use common::storage::UserStorageManager;
use course::{CourseFolderRenderer, PresentationFolderRenderer};
use media_viewer::MediaViewerRenderer;
use site_overview::SiteOverviewRenderer;
use sqlx::SqlitePool;
use workspace_manager::WorkspaceManagerState;

/// Register all built-in folder-type renderers onto `state`.
///
/// Call this before wrapping the state in `Arc`.
pub fn register_all(
    state: &mut WorkspaceManagerState,
    pool: SqlitePool,
    user_storage: UserStorageManager,
) {
    state.register_renderer(Arc::new(BpmnFolderRenderer));
    state.register_renderer(Arc::new(MediaViewerRenderer { pool: pool.clone() }));
    state.register_renderer(Arc::new(CourseFolderRenderer { storage: user_storage.clone(), pool: pool.clone() }));
    state.register_renderer(Arc::new(PresentationFolderRenderer { storage: user_storage, pool }));
    state.register_renderer(Arc::new(SiteOverviewRenderer));
}

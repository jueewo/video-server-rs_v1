pub use js_tool_viewer::{js_tool_viewer_routes, JsToolViewerState};
pub use gallery3d;
pub use publications::{publications_routes, PublicationsState};

use axum::Router;
use common::storage::UserStorageManager;
use db::publications::PublicationRepository;
use db::workspaces::WorkspaceRepository;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

/// Mount all workspace apps onto a single router.
pub fn workspace_app_routes(
    pool: SqlitePool,
    repo: Arc<dyn PublicationRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    storage_base: PathBuf,
    apps_dir: PathBuf,
    user_storage: UserStorageManager,
) -> Router {
    let js_state = Arc::new(JsToolViewerState {
        pool: pool.clone(),
        storage_base: storage_base.clone(),
    });
    let pub_state = Arc::new(PublicationsState {
        repo,
        workspace_repo,
        storage_base,
        apps_dir,
        user_storage,
    });
    Router::new()
        .merge(publications_routes(pub_state))
        .merge(js_tool_viewer_routes(js_state))
        .merge(gallery3d::router(Arc::new(pool)))
}

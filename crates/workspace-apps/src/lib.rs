pub use app_publisher::{app_publisher_routes, AppPublisherState};
pub use js_tool_viewer::{js_tool_viewer_routes, JsToolViewerState};
pub use gallery3d;

use axum::Router;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

/// Mount all workspace apps onto a single router.
pub fn workspace_app_routes(pool: SqlitePool, storage_base: PathBuf, apps_dir: PathBuf) -> Router {
    let app_pub_state = Arc::new(AppPublisherState {
        pool: pool.clone(),
        storage_base: storage_base.clone(),
        apps_dir,
    });
    let js_state = Arc::new(JsToolViewerState {
        pool: pool.clone(),
        storage_base,
    });
    Router::new()
        .merge(app_publisher_routes(app_pub_state))
        .merge(js_tool_viewer_routes(js_state))
        .merge(gallery3d::router(Arc::new(pool)))
}

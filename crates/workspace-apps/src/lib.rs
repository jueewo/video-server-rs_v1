mod pub_app_proxy;

pub use js_tool_viewer::{js_tool_viewer_routes, JsToolViewerState};
pub use gallery3d;
pub use publications::{publications_routes, PublicationsState};

use axum::routing::get;
use axum::Router;
use common::storage::UserStorageManager;
use db::publications::PublicationRepository;
use db::workspaces::WorkspaceRepository;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

/// Combined state for the pub-apps proxy (needs both publication DB and sidecar manager).
pub struct PubAppProxyState {
    pub pub_repo: Arc<dyn PublicationRepository>,
    pub app_runtime: Arc<app_runtime::AppRuntimeState>,
    pub storage_base: PathBuf,
}

/// Mount all workspace apps onto a single router.
pub fn workspace_app_routes(
    pool: SqlitePool,
    repo: Arc<dyn PublicationRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    storage_base: PathBuf,
    apps_dir: PathBuf,
    user_storage: UserStorageManager,
    app_runtime_state: Arc<app_runtime::AppRuntimeState>,
    appstore_registry: Option<Arc<appstore::AppTemplateRegistry>>,
) -> Router {
    let js_state = Arc::new(JsToolViewerState {
        pool: pool.clone(),
        storage_base: storage_base.clone(),
        appstore_registry: appstore_registry.clone(),
    });
    let pub_state = Arc::new(PublicationsState {
        repo: repo.clone(),
        workspace_repo,
        storage_base: storage_base.clone(),
        apps_dir,
        user_storage,
        appstore_registry,
    });
    let pub_proxy_state = Arc::new(PubAppProxyState {
        pub_repo: repo,
        app_runtime: app_runtime_state,
        storage_base,
    });

    // Each sub-router has its own state; merge them together
    let pub_apps_route = Router::new()
        .route(
            "/api/pub-apps/{slug}/{*rest}",
            get(pub_app_proxy::pub_app_proxy_handler)
                .post(pub_app_proxy::pub_app_proxy_handler)
                .put(pub_app_proxy::pub_app_proxy_handler)
                .delete(pub_app_proxy::pub_app_proxy_handler)
                .patch(pub_app_proxy::pub_app_proxy_handler),
        )
        .with_state(pub_proxy_state);

    Router::new()
        .merge(publications_routes(pub_state))
        .merge(js_tool_viewer_routes(js_state))
        .merge(gallery3d::router(Arc::new(pool)))
        .merge(pub_apps_route)
}

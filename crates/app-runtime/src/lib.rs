//! App Runtime — Bun sidecar management for full-stack workspace apps.
//!
//! Apps that include a `server.ts` or `server.js` get a Bun process spawned
//! on demand. The platform proxies API requests to the sidecar and manages
//! its lifecycle (idle timeout, health checks, cleanup on shutdown).

mod db_persist;
mod proxy;
mod sidecar;

pub use sidecar::SidecarManager;

use axum::routing::get;
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Shared state for the app runtime system.
pub struct AppRuntimeState {
    pub sidecar: SidecarManager,
    pub storage_base: PathBuf,
}

impl AppRuntimeState {
    pub fn new(storage_base: PathBuf) -> Self {
        Self {
            sidecar: SidecarManager::new(),
            storage_base,
        }
    }
}

/// Build the router for app runtime API proxy routes.
///
/// All HTTP methods are forwarded to the sidecar.
pub fn app_runtime_routes(state: Arc<AppRuntimeState>) -> Router {
    // Start the idle cleanup background task
    start_cleanup_task(state.clone());

    Router::new()
        .route(
            "/api/apps/{workspace_id}/{*rest}",
            get(proxy::proxy_handler)
                .post(proxy::proxy_handler)
                .put(proxy::proxy_handler)
                .delete(proxy::proxy_handler)
                .patch(proxy::proxy_handler),
        )
        .route(
            "/api/app-db/{workspace_id}/{*folder_path}",
            get(db_persist::db_status_handler).post(db_persist::db_save_handler),
        )
        .with_state(state)
}

/// Spawn background task that stops idle sidecars every 60 seconds.
fn start_cleanup_task(state: Arc<AppRuntimeState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            state
                .sidecar
                .cleanup_idle(Duration::from_secs(300))
                .await;
        }
    });

    info!("App runtime: idle cleanup task started (5 min timeout)");
}

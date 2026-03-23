pub mod cache;
pub mod client;
pub mod models;
pub mod routes;
pub mod server;

use common::storage::UserStorageManager;
use sqlx::SqlitePool;

/// Shared state for federation handlers
pub struct FederationState {
    pub pool: SqlitePool,
    pub storage: UserStorageManager,
    pub storage_dir: String,
    pub server_id: String,
    pub server_name: String,
    pub federation_enabled: bool,
}

pub use routes::{federation_consumer_routes, federation_server_routes};

/// Spawn the background sync task that periodically pulls catalogs from all peers.
/// Runs every `interval_minutes` minutes.
pub fn spawn_sync_task(
    pool: SqlitePool,
    storage_dir: String,
    interval_minutes: u64,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_minutes * 60));
        // Skip the first tick (fires immediately)
        interval.tick().await;

        loop {
            interval.tick().await;
            tracing::info!("Federation: starting periodic catalog sync");

            let peers = sqlx::query_as::<_, models::FederationPeer>(
                "SELECT * FROM federation_peers WHERE status != 'disabled'"
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            for peer in &peers {
                match cache::sync_peer_catalog(&pool, peer, &storage_dir).await {
                    Ok(count) => {
                        tracing::info!("Federation: synced {} items from '{}'", count, peer.display_name);
                    }
                    Err(e) => {
                        tracing::warn!("Federation: failed to sync '{}': {}", peer.display_name, e);
                        let _ = sqlx::query("UPDATE federation_peers SET status = 'error' WHERE id = ?1")
                            .bind(peer.id)
                            .execute(&pool)
                            .await;
                    }
                }
            }
        }
    });
}

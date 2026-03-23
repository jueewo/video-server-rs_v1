pub mod cache;
pub mod client;
pub mod models;
pub mod routes;
pub mod server;

use common::storage::UserStorageManager;
use db::federation::FederationRepository;
use db::media::MediaRepository;
use std::sync::Arc;

/// Shared state for federation handlers
pub struct FederationState {
    pub repo: Arc<dyn FederationRepository>,
    pub media_repo: Arc<dyn MediaRepository>,
    pub storage: UserStorageManager,
    pub storage_dir: String,
    pub server_id: String,
    pub server_name: String,
    pub federation_enabled: bool,
    /// Maximum number of items to cache per peer (0 = unlimited)
    pub max_items_per_peer: i32,
}

pub use routes::{federation_consumer_routes, federation_server_routes};

/// Spawn the background sync task that periodically pulls catalogs from all peers.
/// Runs every `interval_minutes` minutes.
pub fn spawn_sync_task(
    repo: Arc<dyn FederationRepository>,
    storage_dir: String,
    interval_minutes: u64,
    max_items_per_peer: i32,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_minutes * 60));
        // Skip the first tick (fires immediately)
        interval.tick().await;

        loop {
            interval.tick().await;
            tracing::info!("Federation: starting periodic catalog sync");

            let peers = repo.list_active_peers().await.unwrap_or_default();

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            for peer in &peers {
                // Skip peers in backoff — next_retry_at is in the future
                if let Some(ref retry_at) = peer.next_retry_at {
                    if retry_at.as_str() > now.as_str() {
                        tracing::debug!(
                            "Federation: skipping '{}' (backoff until {})",
                            peer.display_name, retry_at
                        );
                        continue;
                    }
                }

                match cache::sync_peer_catalog(repo.as_ref(), peer, &storage_dir, max_items_per_peer).await {
                    Ok(count) => {
                        tracing::info!("Federation: synced {} items from '{}'", count, peer.display_name);
                        // Reset failure counter on success
                        let _ = repo.reset_peer_backoff(peer.id).await;
                    }
                    Err(e) => {
                        let failures = peer.consecutive_failures + 1;
                        // Exponential backoff: 1, 2, 4, 8, 16, 32 intervals (capped)
                        let backoff_multiplier = (1i64 << failures.min(5)) as i64;
                        let backoff_minutes = backoff_multiplier * interval_minutes as i64;

                        tracing::warn!(
                            "Federation: failed to sync '{}' ({} consecutive failures, next retry in {} min): {}",
                            peer.display_name, failures, backoff_minutes, e
                        );

                        let _ = repo.update_peer_sync_failure(peer.id, failures, backoff_minutes as i32).await;
                    }
                }
            }
        }
    });
}

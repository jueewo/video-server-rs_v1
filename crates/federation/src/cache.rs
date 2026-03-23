//! Cache management for remote media metadata and thumbnails

use anyhow::{Context, Result};
use db::federation::{FederationRepository, UpsertRemoteItemRequest};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::client::FederationClient;
use crate::models::{CatalogItem, FederationPeer};

/// Sync a single peer's catalog into our local cache.
/// `max_items` caps how many items to cache (0 = unlimited).
pub async fn sync_peer_catalog(
    repo: &dyn FederationRepository,
    peer: &FederationPeer,
    storage_dir: &str,
    max_items: i32,
) -> Result<i32> {
    let client = FederationClient::new(&peer.server_url, &peer.api_key);

    // Verify the peer is reachable
    let manifest = client.fetch_manifest().await.context("Peer unreachable")?;
    info!(
        "Syncing from peer '{}' ({}): {} items",
        peer.display_name, manifest.server_id, manifest.catalog_count
    );

    // Update peer's server_id if it changed (first sync)
    if peer.server_id != manifest.server_id {
        repo.update_peer_server_id(peer.id, &manifest.server_id).await?;
    }

    let mut total_synced = 0;
    let page_size = 100;
    let mut page = 1;

    loop {
        let catalog = client.fetch_catalog(page, page_size).await?;
        if catalog.items.is_empty() {
            break;
        }

        for item in &catalog.items {
            upsert_remote_item(repo, &peer.server_id, item).await?;

            // Download thumbnail
            let thumb_dir = federation_cache_thumbnail_dir(storage_dir, &peer.server_id);
            let thumb_path = thumb_dir.join(format!("{}_thumb.webp", item.slug));
            if !thumb_path.exists() {
                match client.fetch_thumbnail(&item.slug, &thumb_path).await {
                    Ok(_) => {
                        repo.mark_thumbnail_cached(&peer.server_id, &item.slug).await?;
                    }
                    Err(e) => {
                        warn!("Failed to cache thumbnail for {}: {}", item.slug, e);
                    }
                }
            }

            total_synced += 1;

            // Enforce max items cap
            if max_items > 0 && total_synced >= max_items {
                info!("Reached max_items_per_peer limit ({}), stopping sync", max_items);
                break;
            }
        }

        if max_items > 0 && total_synced >= max_items {
            break;
        }

        if (page * page_size) as i64 >= catalog.total {
            break;
        }
        page += 1;
    }

    // Update peer status and reset failure tracking
    repo.update_peer_sync_success(peer.id, total_synced).await?;

    info!("Synced {} items from peer '{}'", total_synced, peer.display_name);
    Ok(total_synced)
}

async fn upsert_remote_item(
    repo: &dyn FederationRepository,
    origin_server: &str,
    item: &CatalogItem,
) -> Result<()> {
    repo.upsert_remote_item(&UpsertRemoteItemRequest {
        origin_server,
        remote_slug: &item.slug,
        media_type: &item.media_type,
        title: &item.title,
        description: item.description.as_deref(),
        filename: item.filename.as_deref(),
        mime_type: item.mime_type.as_deref(),
        file_size: item.file_size,
    }).await?;

    Ok(())
}

/// Get the cache directory for a peer's thumbnails
pub fn federation_cache_thumbnail_dir(storage_dir: &str, server_id: &str) -> PathBuf {
    Path::new(storage_dir)
        .join("federation_cache")
        .join(server_id)
        .join("thumbnails")
}

/// Get the cache directory for a peer's media content
pub fn federation_cache_media_dir(storage_dir: &str, server_id: &str, slug: &str) -> PathBuf {
    Path::new(storage_dir)
        .join("federation_cache")
        .join(server_id)
        .join("media")
        .join(slug)
}

/// Remove all cached data for a peer
pub async fn clear_peer_cache(
    repo: &dyn FederationRepository,
    server_id: &str,
    storage_dir: &str,
) -> Result<()> {
    // Remove from database
    repo.clear_peer_cache(server_id).await?;

    // Remove from filesystem
    let cache_dir = Path::new(storage_dir)
        .join("federation_cache")
        .join(server_id);
    if cache_dir.exists() {
        tokio::fs::remove_dir_all(&cache_dir).await?;
    }

    info!("Cleared cache for peer {}", server_id);
    Ok(())
}

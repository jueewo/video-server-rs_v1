//! Cache management for remote media metadata and thumbnails

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::client::FederationClient;
use crate::models::{CatalogItem, FederationPeer};

/// Sync a single peer's catalog into our local cache.
/// `max_items` caps how many items to cache (0 = unlimited).
pub async fn sync_peer_catalog(
    pool: &SqlitePool,
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
        sqlx::query("UPDATE federation_peers SET server_id = ?1 WHERE id = ?2")
            .bind(&manifest.server_id)
            .bind(peer.id)
            .execute(pool)
            .await?;
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
            upsert_remote_item(pool, &peer.server_id, item).await?;

            // Download thumbnail
            let thumb_dir = federation_cache_thumbnail_dir(storage_dir, &peer.server_id);
            let thumb_path = thumb_dir.join(format!("{}_thumb.webp", item.slug));
            if !thumb_path.exists() {
                match client.fetch_thumbnail(&item.slug, &thumb_path).await {
                    Ok(_) => {
                        sqlx::query(
                            "UPDATE remote_media_cache SET thumbnail_cached = 1 \
                             WHERE origin_server = ?1 AND remote_slug = ?2"
                        )
                        .bind(&peer.server_id)
                        .bind(&item.slug)
                        .execute(pool)
                        .await?;
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
    sqlx::query(
        "UPDATE federation_peers SET last_synced_at = datetime('now'), status = 'online', item_count = ?1, \
         consecutive_failures = 0, next_retry_at = NULL WHERE id = ?2"
    )
    .bind(total_synced)
    .bind(peer.id)
    .execute(pool)
    .await?;

    info!("Synced {} items from peer '{}'", total_synced, peer.display_name);
    Ok(total_synced)
}

async fn upsert_remote_item(
    pool: &SqlitePool,
    origin_server: &str,
    item: &CatalogItem,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO remote_media_cache (origin_server, remote_slug, media_type, title, description, filename, mime_type, file_size, cached_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now')) \
         ON CONFLICT(origin_server, remote_slug) DO UPDATE SET \
           title = excluded.title, \
           description = excluded.description, \
           filename = excluded.filename, \
           mime_type = excluded.mime_type, \
           file_size = excluded.file_size, \
           updated_at = datetime('now')"
    )
    .bind(origin_server)
    .bind(&item.slug)
    .bind(&item.media_type)
    .bind(&item.title)
    .bind(&item.description)
    .bind(&item.filename)
    .bind(&item.mime_type)
    .bind(item.file_size)
    .execute(pool)
    .await?;

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
    pool: &SqlitePool,
    server_id: &str,
    storage_dir: &str,
) -> Result<()> {
    // Remove from database
    sqlx::query("DELETE FROM remote_media_cache WHERE origin_server = ?1")
        .bind(server_id)
        .execute(pool)
        .await?;

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

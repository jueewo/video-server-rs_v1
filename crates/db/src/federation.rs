//! Federation repository trait and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};

/// A configured remote server (peer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPeer {
    pub id: i32,
    pub server_id: String,
    pub server_url: String,
    pub display_name: String,
    pub api_key: String,
    pub last_synced_at: Option<String>,
    pub status: String,
    pub item_count: i32,
    pub created_at: String,
    pub consecutive_failures: i32,
    pub next_retry_at: Option<String>,
    pub tenant_id: String,
}

/// Cached metadata from a remote peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMediaItem {
    pub id: i32,
    pub origin_server: String,
    pub remote_slug: String,
    pub media_type: String,
    pub title: String,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub thumbnail_cached: i32,
    pub cached_at: String,
    pub updated_at: Option<String>,
}

/// Parameters for upserting a remote media cache entry.
#[derive(Debug)]
pub struct UpsertRemoteItemRequest<'a> {
    pub origin_server: &'a str,
    pub remote_slug: &'a str,
    pub media_type: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub filename: Option<&'a str>,
    pub mime_type: Option<&'a str>,
    pub file_size: Option<i64>,
    pub tenant_id: &'a str,
}

#[async_trait::async_trait]
pub trait FederationRepository: Send + Sync {
    // ── Peers ───────────────────────────────────────────────────────

    /// List all peers ordered by display_name.
    async fn list_peers(&self, tenant_id: &str) -> Result<Vec<FederationPeer>, DbError>;

    /// List all non-disabled peers (for sync).
    async fn list_active_peers(&self, tenant_id: &str) -> Result<Vec<FederationPeer>, DbError>;

    /// Get a peer by its database ID.
    async fn get_peer_by_id(&self, id: i32) -> Result<Option<FederationPeer>, DbError>;

    /// Get a peer by its server_id.
    async fn get_peer_by_server_id(&self, server_id: &str) -> Result<Option<FederationPeer>, DbError>;

    /// Insert a new peer.
    async fn insert_peer(
        &self,
        server_id: &str,
        server_url: &str,
        display_name: &str,
        api_key: &str,
        tenant_id: &str,
    ) -> Result<(), DbError>;

    /// Delete a peer by ID.
    async fn delete_peer(&self, id: i32) -> Result<(), DbError>;

    /// Update a peer's server_id (learned during first sync).
    async fn update_peer_server_id(&self, peer_id: i32, server_id: &str) -> Result<(), DbError>;

    /// Update peer after successful sync (status=online, item_count, reset backoff).
    async fn update_peer_sync_success(&self, peer_id: i32, item_count: i32) -> Result<(), DbError>;

    /// Update peer after failed sync (status=error, increment failures, set backoff).
    async fn update_peer_sync_failure(
        &self,
        peer_id: i32,
        consecutive_failures: i32,
        backoff_minutes: i32,
    ) -> Result<(), DbError>;

    /// Set peer status (e.g. 'syncing').
    async fn set_peer_status(&self, peer_id: i32, status: &str) -> Result<(), DbError>;

    /// Reset peer backoff counters.
    async fn reset_peer_backoff(&self, peer_id: i32) -> Result<(), DbError>;

    /// Increment peer failure counter and set status to 'error'.
    async fn increment_peer_failures(&self, peer_id: i32) -> Result<(), DbError>;

    // ── Remote media cache ──────────────────────────────────────────

    /// Insert or update a cached remote media item.
    async fn upsert_remote_item(&self, req: &UpsertRemoteItemRequest<'_>) -> Result<(), DbError>;

    /// Mark a remote item's thumbnail as cached.
    async fn mark_thumbnail_cached(&self, origin_server: &str, slug: &str) -> Result<(), DbError>;

    /// List remote media items, optionally filtered by media_type.
    async fn list_remote_media(
        &self,
        origin_server: &str,
        media_type: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<RemoteMediaItem>, DbError>;

    /// Count remote media items, optionally filtered by media_type.
    async fn count_remote_media(
        &self,
        origin_server: &str,
        media_type: Option<&str>,
    ) -> Result<i64, DbError>;

    /// Get a single remote media item by origin + slug.
    async fn get_remote_item(
        &self,
        origin_server: &str,
        slug: &str,
    ) -> Result<Option<RemoteMediaItem>, DbError>;

    /// Delete all cached items for a peer's origin_server.
    async fn clear_peer_cache(&self, origin_server: &str) -> Result<(), DbError>;
}

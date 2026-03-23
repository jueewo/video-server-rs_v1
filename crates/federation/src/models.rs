use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A configured remote server (peer)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FederationPeer {
    pub id: i32,
    pub server_id: String,
    pub server_url: String,
    pub display_name: String,
    pub api_key: String,
    pub last_synced_at: Option<String>,
    pub status: String, // online, offline, syncing, error
    pub item_count: i32,
    pub created_at: String,
}

/// Subset for creation
#[derive(Debug, Deserialize)]
pub struct CreatePeerRequest {
    pub server_url: String,
    pub display_name: String,
    pub api_key: String,
}

/// Cached metadata from a remote peer
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub thumbnail_cached: i32, // 0 or 1
    pub cached_at: String,
    pub updated_at: Option<String>,
}

/// Server manifest — returned by origin to identify itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerManifest {
    pub server_id: String,
    pub server_name: String,
    pub version: String,
    pub catalog_count: i64,
    pub federation_api_version: String,
}

/// A single catalog entry returned by the origin server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    pub slug: String,
    pub media_type: String,
    pub title: String,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// Paginated catalog response from the origin server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub items: Vec<CatalogItem>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

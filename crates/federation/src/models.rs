use serde::{Deserialize, Serialize};

// Re-export domain types from the db crate
pub use db::federation::{FederationPeer, RemoteMediaItem, UpsertRemoteItemRequest};

/// Subset for creation
#[derive(Debug, Deserialize)]
pub struct CreatePeerRequest {
    pub server_url: String,
    pub display_name: String,
    pub api_key: String,
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

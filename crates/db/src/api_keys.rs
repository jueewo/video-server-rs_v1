//! API key domain — types and repository trait.

use serde::{Deserialize, Serialize};

use crate::DbError;

// ============================================================================
// Domain types
// ============================================================================

/// API Key metadata stored in database (without the raw key).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub user_id: String,
    pub key_prefix: String,
    pub name: String,
    pub description: Option<String>,
    pub scopes: String,
    pub last_used_at: Option<String>,
    pub usage_count: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub is_active: bool,
}

impl ApiKey {
    pub fn get_scopes(&self) -> Vec<String> {
        serde_json::from_str(&self.scopes).unwrap_or_default()
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.get_scopes().contains(&scope.to_string())
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = &self.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                return expiry < chrono::Utc::now();
            }
        }
        false
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }
}

/// Request to create a new API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}

/// Response when creating an API key (includes full key — only shown once).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub key: String,
    pub api_key: ApiKey,
}

/// Request to update API key metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<String>,
}

// ============================================================================
// Repository trait
// ============================================================================

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Create a new API key. Returns the full key string + metadata.
    /// The key_hash and key_prefix are derived from the generated key.
    async fn create_api_key(
        &self,
        user_id: &str,
        key: &str,
        key_hash: &str,
        key_prefix: &str,
        request: &CreateApiKeyRequest,
    ) -> Result<ApiKey, DbError>;

    /// Look up an API key by its hash (for authentication).
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DbError>;

    /// Get an API key by ID with ownership check.
    async fn get_api_key_by_id(&self, key_id: i32, user_id: &str) -> Result<Option<ApiKey>, DbError>;

    /// List all API keys for a user.
    async fn list_user_api_keys(&self, user_id: &str) -> Result<Vec<ApiKey>, DbError>;

    /// Revoke (soft delete) an API key.
    async fn revoke_api_key(&self, key_id: i32, user_id: &str) -> Result<bool, DbError>;

    /// Update API key metadata.
    async fn update_api_key(
        &self,
        key_id: i32,
        user_id: &str,
        request: &UpdateApiKeyRequest,
    ) -> Result<Option<ApiKey>, DbError>;

    /// Update last_used_at timestamp and increment usage_count.
    async fn update_last_used(&self, key_id: i32) -> Result<(), DbError>;
}

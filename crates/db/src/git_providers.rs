//! Git provider domain — types and repository trait.

use serde::{Deserialize, Serialize};

use crate::DbError;

// ============================================================================
// Domain types
// ============================================================================

/// Git provider stored in database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitProvider {
    pub id: i32,
    pub user_id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub token_encrypted: String,
    pub token_prefix: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Safe view of a provider (no encrypted tokens).
#[derive(Debug, Clone, Serialize)]
pub struct GitProviderSafe {
    pub id: i32,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub token_prefix: String,
    pub is_default: bool,
}

impl From<&GitProvider> for GitProviderSafe {
    fn from(p: &GitProvider) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            provider_type: p.provider_type.clone(),
            base_url: p.base_url.clone(),
            token_prefix: p.token_prefix.clone(),
            is_default: p.is_default,
        }
    }
}

/// Request to create a new git provider (token encrypted externally).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateGitProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub token_encrypted: String,
    pub token_prefix: String,
    pub is_default: bool,
}

/// Request to update a git provider.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGitProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub is_default: bool,
    /// If Some, re-encrypt with new values.
    pub new_token_encrypted: Option<String>,
    pub new_token_prefix: Option<String>,
}

// ============================================================================
// Repository trait
// ============================================================================

#[async_trait::async_trait]
pub trait GitProviderRepository: Send + Sync {
    /// Insert a new git provider.
    async fn create_provider(
        &self,
        user_id: &str,
        req: &CreateGitProviderRequest,
    ) -> Result<GitProvider, DbError>;

    /// Get a provider by ID with ownership check.
    async fn get_provider_by_id(&self, id: i32, user_id: &str) -> Result<Option<GitProvider>, DbError>;

    /// Get a provider by name for a user.
    async fn get_provider_by_name(&self, user_id: &str, name: &str) -> Result<Option<GitProvider>, DbError>;

    /// Get the default provider for a user.
    async fn get_default_provider(&self, user_id: &str) -> Result<Option<GitProvider>, DbError>;

    /// List all providers for a user.
    async fn list_providers(&self, user_id: &str) -> Result<Vec<GitProvider>, DbError>;

    /// Update a provider.
    async fn update_provider(
        &self,
        id: i32,
        user_id: &str,
        req: &UpdateGitProviderRequest,
    ) -> Result<bool, DbError>;

    /// Delete a provider.
    async fn delete_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError>;

    /// Set a provider as default (unsets others).
    async fn set_default_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError>;
}

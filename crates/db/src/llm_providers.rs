//! LLM provider domain — types and repository trait.

use serde::{Deserialize, Serialize};

use crate::DbError;

// ============================================================================
// Domain types
// ============================================================================

/// LLM provider stored in database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub id: i32,
    pub user_id: String,
    pub name: String,
    pub provider: String,
    pub api_url: String,
    pub api_key_encrypted: String,
    pub api_key_prefix: String,
    pub default_model: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Safe view of a provider (no encrypted keys).
#[derive(Debug, Clone, Serialize)]
pub struct LlmProviderSafe {
    pub id: i32,
    pub name: String,
    pub provider: String,
    pub api_url: String,
    pub api_key_prefix: String,
    pub default_model: String,
    pub is_default: bool,
}

impl From<&LlmProvider> for LlmProviderSafe {
    fn from(p: &LlmProvider) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            provider: p.provider.clone(),
            api_url: p.api_url.clone(),
            api_key_prefix: p.api_key_prefix.clone(),
            default_model: p.default_model.clone(),
            is_default: p.is_default,
        }
    }
}

/// Request to create a new LLM provider (encrypted key handled externally).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateLlmProviderRequest {
    pub name: String,
    pub provider: String,
    pub api_url: String,
    pub api_key_encrypted: String,
    pub api_key_prefix: String,
    pub default_model: String,
    pub is_default: bool,
}

/// Usage summary aggregated by provider and model.
#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub provider_name: String,
    pub model: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub request_count: i64,
}

// ============================================================================
// Repository trait
// ============================================================================

#[async_trait::async_trait]
pub trait LlmProviderRepository: Send + Sync {
    /// Insert a new LLM provider. Unsets other defaults if is_default.
    async fn create_provider(
        &self,
        user_id: &str,
        req: &CreateLlmProviderRequest,
    ) -> Result<LlmProvider, DbError>;

    /// Get a provider by ID with ownership check.
    async fn get_provider_by_id(&self, id: i32, user_id: &str) -> Result<Option<LlmProvider>, DbError>;

    /// Get a provider by name for a user.
    async fn get_provider_by_name(&self, user_id: &str, name: &str) -> Result<Option<LlmProvider>, DbError>;

    /// Get the default provider for a user.
    async fn get_default_provider(&self, user_id: &str) -> Result<Option<LlmProvider>, DbError>;

    /// List all providers for a user.
    async fn list_providers(&self, user_id: &str) -> Result<Vec<LlmProvider>, DbError>;

    /// Update a provider. If has_new_key, also updates encrypted key and prefix.
    async fn update_provider(
        &self,
        id: i32,
        user_id: &str,
        name: &str,
        provider: &str,
        api_url: &str,
        default_model: &str,
        is_default: bool,
        new_key_encrypted: Option<&str>,
        new_key_prefix: Option<&str>,
    ) -> Result<bool, DbError>;

    /// Delete a provider.
    async fn delete_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError>;

    /// Set a provider as default (unsets others).
    async fn set_default_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError>;

    /// Log usage of an LLM call.
    async fn log_usage(
        &self,
        user_id: &str,
        provider_name: &str,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) -> Result<(), DbError>;

    /// Get aggregated usage summary for a user.
    async fn get_user_usage_summary(&self, user_id: &str) -> Result<Vec<UsageSummary>, DbError>;
}

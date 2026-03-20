pub mod crypto;
pub mod db;
pub mod providers;
pub mod routes;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::PathBuf;

// -------------------------------
// Core Types
// -------------------------------

/// LLM provider stored in database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LlmProvider {
    pub id: i32,
    pub user_id: String,
    pub name: String,
    pub provider: String,          // "anthropic" | "openai-compatible"
    pub api_url: String,
    pub api_key_encrypted: String,
    pub api_key_prefix: String,    // first 8 chars for safe display
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

/// Request to create a new provider.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub provider: String,
    pub api_url: String,
    pub api_key: String,           // plaintext, will be encrypted
    pub default_model: String,
    pub is_default: bool,
}

/// Request to update an existing provider.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: String,
    pub provider: String,
    pub api_url: String,
    pub api_key: Option<String>,        // if Some & non-empty, re-encrypt; if None/empty, keep existing
    pub default_model: String,
    pub is_default: bool,
}

/// Chat message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat request body.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub provider_name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    // Folder-level override fields
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub folder_path: Option<String>,
}

fn default_max_tokens() -> u32 {
    4096
}

// -------------------------------
// State
// -------------------------------

/// Usage stats returned with SSE done event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub provider_id: i32,
    pub provider_name: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone)]
pub struct LlmProviderState {
    pub pool: SqlitePool,
    pub http_client: Client,
    /// Root of workspace storage — for reading workspace.yaml metadata.
    pub storage_root: Option<PathBuf>,
}

impl LlmProviderState {
    pub fn new(pool: SqlitePool) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client for LLM provider");

        Self { pool, http_client, storage_root: None }
    }

    pub fn with_storage(mut self, storage_root: PathBuf) -> Self {
        self.storage_root = Some(storage_root);
        self
    }
}

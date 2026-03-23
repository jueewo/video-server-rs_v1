pub mod crypto;
pub mod db;
pub mod providers;
pub mod routes;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

// Re-export domain types from the db crate
pub use ::db::llm_providers::{LlmProvider, LlmProviderSafe, CreateLlmProviderRequest};

// -------------------------------
// User-facing request types (plaintext keys — encryption happens in db.rs)
// -------------------------------

/// Request to create a new provider (user-facing, plaintext API key).
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
    pub repo: Arc<dyn ::db::llm_providers::LlmProviderRepository>,
    pub http_client: Client,
    /// Root of workspace storage — for reading workspace.yaml metadata.
    pub storage_root: Option<PathBuf>,
}

impl LlmProviderState {
    pub fn new(repo: Arc<dyn ::db::llm_providers::LlmProviderRepository>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client for LLM provider");

        Self { repo, http_client, storage_root: None }
    }

    pub fn with_storage(mut self, storage_root: PathBuf) -> Self {
        self.storage_root = Some(storage_root);
        self
    }
}

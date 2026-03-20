pub mod crypto;
pub mod db;
pub mod api;
pub mod routes;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// -------------------------------
// Core Types
// -------------------------------

/// Git provider stored in database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GitProvider {
    pub id: i32,
    pub user_id: String,
    pub name: String,
    pub provider_type: String,      // "forgejo" | "gitea" | "github" | "gitlab"
    pub base_url: String,           // e.g. "https://git.appkask.com"
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

/// Request to create a new git provider.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateGitProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub token: String,
    pub is_default: bool,
}

/// Request to update an existing git provider.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGitProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub token: Option<String>,      // if Some & non-empty, re-encrypt
    pub is_default: bool,
}

// -------------------------------
// State
// -------------------------------

#[derive(Clone)]
pub struct GitProviderState {
    pub pool: SqlitePool,
    pub http_client: Client,
}

impl GitProviderState {
    pub fn new(pool: SqlitePool) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client for git provider");

        Self { pool, http_client }
    }
}

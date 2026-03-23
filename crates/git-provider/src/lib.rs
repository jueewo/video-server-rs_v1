pub mod crypto;
pub mod db;
pub mod api;
pub mod routes;

use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

// Re-export domain types from the db crate
pub use ::db::git_providers::{GitProvider, GitProviderSafe};

// -------------------------------
// User-facing request types (with plain token)
// -------------------------------

/// User-facing request to create a new git provider (plain token).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateGitProviderInput {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub token: String,
    pub is_default: bool,
}

/// User-facing request to update an existing git provider (plain token).
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGitProviderInput {
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
    pub repo: Arc<dyn ::db::git_providers::GitProviderRepository>,
    pub http_client: Client,
}

impl GitProviderState {
    pub fn new(repo: Arc<dyn ::db::git_providers::GitProviderRepository>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client for git provider");

        Self { repo, http_client }
    }
}

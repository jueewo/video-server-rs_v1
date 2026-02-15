pub mod db;
pub mod generator;
pub mod middleware;
pub mod routes;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// -------------------------------
// Core Types
// -------------------------------

/// API Key metadata stored in database (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: i32,
    pub user_id: String,
    pub key_prefix: String,  // For display (e.g., "ak_live_abc1")
    pub name: String,
    pub description: Option<String>,
    pub scopes: String,      // JSON array as string
    pub last_used_at: Option<String>,
    pub usage_count: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub is_active: bool,
}

impl ApiKey {
    /// Parse scopes from JSON string to Vec<String>
    pub fn get_scopes(&self) -> Vec<String> {
        serde_json::from_str(&self.scopes).unwrap_or_default()
    }

    /// Check if key has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.get_scopes().contains(&scope.to_string())
    }

    /// Check if key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = &self.expires_at {
            // Parse ISO 8601 timestamp and compare with now
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                return expiry < chrono::Utc::now();
            }
        }
        false
    }

    /// Check if key is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }
}

/// Request to create a new API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,  // ISO 8601 format
}

/// Response when creating an API key (includes full key - only shown once!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub key: String,  // Full key, only returned once at creation
    pub api_key: ApiKey,  // Metadata
}

/// Request to update API key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    // Note: scopes cannot be changed for security reasons
}

/// Available scopes for API keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Read,
    Write,
    Delete,
    Admin,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Read => "read",
            Scope::Write => "write",
            Scope::Delete => "delete",
            Scope::Admin => "admin",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(Scope::Read),
            "write" => Some(Scope::Write),
            "delete" => Some(Scope::Delete),
            "admin" => Some(Scope::Admin),
            _ => None,
        }
    }

    /// Get all available scopes
    pub fn all() -> Vec<Scope> {
        vec![Scope::Read, Scope::Write, Scope::Delete, Scope::Admin]
    }
}

/// Validate scope list
pub fn validate_scopes(scopes: &[String]) -> Result<()> {
    if scopes.is_empty() {
        return Err(anyhow::anyhow!("At least one scope is required"));
    }

    for scope in scopes {
        if Scope::from_str(scope).is_none() {
            return Err(anyhow::anyhow!("Invalid scope: {}. Valid scopes: read, write, delete, admin", scope));
        }
    }

    Ok(())
}

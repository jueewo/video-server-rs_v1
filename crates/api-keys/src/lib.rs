pub mod db;
pub mod generator;
pub mod middleware;
pub mod routes;

use anyhow::Result;

// Re-export domain types from the db crate
pub use ::db::api_keys::{ApiKey, ApiKeyResponse, CreateApiKeyRequest, UpdateApiKeyRequest};

// ============================================================================
// Business logic (not DB-related)
// ============================================================================

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
            return Err(anyhow::anyhow!(
                "Invalid scope: {}. Valid scopes: read, write, delete, admin",
                scope
            ));
        }
    }

    Ok(())
}

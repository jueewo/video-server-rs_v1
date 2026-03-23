//! Vault repository trait and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};

/// A storage vault record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageVault {
    pub vault_id: String,
    pub user_id: String,
    pub vault_name: String,
    pub is_default: bool,
    pub created_at: String,
}

/// Parameters for inserting a new vault.
#[derive(Debug)]
pub struct InsertVaultRequest<'a> {
    pub vault_id: &'a str,
    pub user_id: &'a str,
    pub vault_name: &'a str,
    pub is_default: bool,
    pub created_at: &'a str,
}

#[async_trait::async_trait]
pub trait VaultRepository: Send + Sync {
    /// Get the default vault_id for a user. Returns None if no default vault.
    async fn get_default_vault_id(&self, user_id: &str) -> Result<Option<String>, DbError>;

    /// Check whether a vault_id already exists.
    async fn vault_id_exists(&self, vault_id: &str) -> Result<bool, DbError>;

    /// Count how many vaults a user owns.
    async fn count_user_vaults(&self, user_id: &str) -> Result<i64, DbError>;

    /// Insert a new vault.
    async fn insert_vault(&self, req: &InsertVaultRequest<'_>) -> Result<(), DbError>;

    /// List all vaults for a user, ordered by is_default DESC, created_at ASC.
    async fn list_user_vaults(&self, user_id: &str) -> Result<Vec<StorageVault>, DbError>;

    /// Count media items in a vault.
    async fn count_vault_media(&self, vault_id: &str) -> Result<i64, DbError>;

    /// Get the owner (user_id) of a vault. Returns None if vault doesn't exist.
    async fn get_vault_owner(&self, vault_id: &str) -> Result<Option<String>, DbError>;

    /// Update a vault's name.
    async fn update_vault_name(&self, vault_id: &str, name: &str) -> Result<(), DbError>;

    /// Set a vault as default for its owner (unsets all others in a transaction).
    async fn set_default_vault(&self, user_id: &str, vault_id: &str) -> Result<(), DbError>;

    /// Delete a vault. Returns true if a row was deleted.
    async fn delete_vault(&self, vault_id: &str, user_id: &str) -> Result<bool, DbError>;
}

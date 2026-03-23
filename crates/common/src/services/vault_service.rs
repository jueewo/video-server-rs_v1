//! Vault service for managing user storage vaults
//!
//! Phase 4.5: Provides utilities for vault creation and retrieval

use crate::storage::{generate_vault_id, UserStorageManager};
use anyhow::{Context, Result};
use db::vaults::{InsertVaultRequest, VaultRepository};
use tracing::info;

/// Get or create a default vault for a user
///
/// This function:
/// 1. Checks if user has a default vault in database
/// 2. If not, creates a new vault with random ID
/// 3. Ensures vault directories exist on filesystem
/// 4. Returns the vault_id
pub async fn get_or_create_default_vault(
    repo: &dyn VaultRepository,
    storage: &UserStorageManager,
    user_id: &str,
) -> Result<String> {
    // Check if user already has a default vault
    let existing_vault = repo
        .get_default_vault_id(user_id)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to query existing vault")?;

    if let Some(vault_id) = existing_vault {
        return Ok(vault_id);
    }

    // Create new vault
    let vault_id = generate_vault_id();
    info!(
        "Creating new default vault for user {}: {}",
        user_id, vault_id
    );

    // Insert into database
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    repo.insert_vault(&InsertVaultRequest {
        vault_id: &vault_id,
        user_id,
        vault_name: &format!("{}'s Media Vault", user_id),
        is_default: true,
        created_at: &now,
    })
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))
    .context("Failed to create vault in database")?;

    // Ensure vault directories exist
    storage
        .ensure_vault_storage(&vault_id)
        .context("Failed to create vault directories")?;

    info!("Created vault {} for user {}", vault_id, user_id);
    Ok(vault_id)
}

/// Get all vaults for a user
pub async fn get_user_vaults(
    repo: &dyn VaultRepository,
    user_id: &str,
) -> Result<Vec<(String, String, bool)>> {
    let vaults = repo
        .list_user_vaults(user_id)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to query user vaults")?;

    Ok(vaults
        .into_iter()
        .map(|v| (v.vault_id, v.vault_name, v.is_default))
        .collect())
}

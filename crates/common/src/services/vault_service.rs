//! Vault service for managing user storage vaults
//!
//! Phase 4.5: Provides utilities for vault creation and retrieval

use crate::storage::{generate_vault_id, UserStorageManager};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tracing::info;

/// Get or create a default vault for a user
///
/// This function:
/// 1. Checks if user has a default vault in database
/// 2. If not, creates a new vault with random ID
/// 3. Ensures vault directories exist on filesystem
/// 4. Returns the vault_id
pub async fn get_or_create_default_vault(
    pool: &SqlitePool,
    storage: &UserStorageManager,
    user_id: &str,
) -> Result<String> {
    // Check if user already has a default vault
    let existing_vault: Option<String> = sqlx::query_scalar(
        "SELECT vault_id FROM storage_vaults WHERE user_id = ? AND is_default = 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .context("Failed to query existing vault")?;

    if let Some(vault_id) = existing_vault {
        return Ok(vault_id);
    }

    // Create new vault
    let vault_id = generate_vault_id();
    info!("Creating new default vault for user {}: {}", user_id, vault_id);

    // Insert into database
    sqlx::query(
        "INSERT INTO storage_vaults (vault_id, user_id, vault_name, is_default) VALUES (?, ?, ?, 1)",
    )
    .bind(&vault_id)
    .bind(user_id)
    .bind("Default Vault")
    .execute(pool)
    .await
    .context("Failed to create vault in database")?;

    // Ensure vault directories exist
    storage
        .ensure_vault_storage(&vault_id)
        .context("Failed to create vault directories")?;

    info!("Created vault {} for user {}", vault_id, user_id);
    Ok(vault_id)
}

/// Get all vaults for a user
pub async fn get_user_vaults(pool: &SqlitePool, user_id: &str) -> Result<Vec<(String, String, bool)>> {
    let vaults: Vec<(String, String, i32)> = sqlx::query_as(
        "SELECT vault_id, vault_name, is_default FROM storage_vaults WHERE user_id = ? ORDER BY is_default DESC, created_at ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .context("Failed to query user vaults")?;

    Ok(vaults
        .into_iter()
        .map(|(vault_id, vault_name, is_default)| (vault_id, vault_name, is_default == 1))
        .collect())
}

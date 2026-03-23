//! API key service functions — business logic + repository calls.

use crate::{generator, ApiKey, ApiKeyResponse, CreateApiKeyRequest, UpdateApiKeyRequest};
use anyhow::{anyhow, Result};
use ::db::api_keys::ApiKeyRepository;
use tracing::{error, info};

/// Create a new API key for a user.
///
/// Generates a secure random key, hashes it, and stores it via the repository.
/// The full key is returned in the response — this is the ONLY time it will be visible!
pub async fn create_api_key(
    repo: &dyn ApiKeyRepository,
    user_id: &str,
    request: CreateApiKeyRequest,
) -> Result<ApiKeyResponse> {
    crate::validate_scopes(&request.scopes)?;

    let key = generator::generate_live_key();
    let key_hash = generator::hash_api_key(&key);
    let key_prefix = generator::extract_prefix(&key);

    let api_key = repo
        .create_api_key(user_id, &key, &key_hash, &key_prefix, &request)
        .await
        .map_err(|e| anyhow!("{e}"))?;

    info!(
        event = "api_key_created",
        user_id = %user_id,
        key_id = api_key.id,
        key_prefix = %key_prefix,
        name = %request.name,
        "API key created successfully"
    );

    Ok(ApiKeyResponse { key, api_key })
}

/// Get an API key by ID (with ownership check).
pub async fn get_api_key_by_id(
    repo: &dyn ApiKeyRepository,
    key_id: i32,
    user_id: &str,
) -> Result<Option<ApiKey>> {
    repo.get_api_key_by_id(key_id, user_id)
        .await
        .map_err(|e| anyhow!("{e}"))
}

/// List all API keys for a user.
pub async fn list_user_api_keys(
    repo: &dyn ApiKeyRepository,
    user_id: &str,
) -> Result<Vec<ApiKey>> {
    repo.list_user_api_keys(user_id)
        .await
        .map_err(|e| anyhow!("{e}"))
}

/// Revoke (soft delete) an API key.
pub async fn revoke_api_key(
    repo: &dyn ApiKeyRepository,
    key_id: i32,
    user_id: &str,
) -> Result<bool> {
    let revoked = repo
        .revoke_api_key(key_id, user_id)
        .await
        .map_err(|e| anyhow!("{e}"))?;

    if revoked {
        info!(
            event = "api_key_revoked",
            user_id = %user_id,
            key_id = key_id,
            "API key revoked successfully"
        );
    }

    Ok(revoked)
}

/// Update API key metadata.
pub async fn update_api_key(
    repo: &dyn ApiKeyRepository,
    key_id: i32,
    user_id: &str,
    request: UpdateApiKeyRequest,
) -> Result<Option<ApiKey>> {
    let result = repo
        .update_api_key(key_id, user_id, &request)
        .await
        .map_err(|e| anyhow!("{e}"))?;

    if result.is_some() {
        info!(
            event = "api_key_updated",
            user_id = %user_id,
            key_id = key_id,
            "API key metadata updated"
        );
    }

    Ok(result)
}

/// Validate an API key string and return the associated ApiKey if valid.
///
/// 1. Hashes the provided key
/// 2. Looks it up in the database
/// 3. Checks if it's active and not expired
/// 4. Updates last_used_at and usage_count if valid
pub async fn validate_api_key(
    repo: &dyn ApiKeyRepository,
    key_str: &str,
) -> Result<Option<ApiKey>> {
    let key_hash = generator::hash_api_key(key_str);

    let api_key = match repo
        .get_api_key_by_hash(&key_hash)
        .await
        .map_err(|e| anyhow!("{e}"))?
    {
        Some(key) => key,
        None => {
            error!(
                event = "api_key_validation_failed",
                reason = "not_found",
                "API key not found in database"
            );
            return Ok(None);
        }
    };

    if !api_key.is_valid() {
        error!(
            event = "api_key_validation_failed",
            reason = if !api_key.is_active { "inactive" } else { "expired" },
            key_id = api_key.id,
            "API key is invalid"
        );
        return Ok(None);
    }

    if let Err(e) = repo.update_last_used(api_key.id).await {
        error!(
            event = "api_key_usage_update_failed",
            error = %e,
            key_id = api_key.id,
            "Failed to update API key usage stats"
        );
    }

    info!(
        event = "api_key_validated",
        key_id = api_key.id,
        user_id = %api_key.user_id,
        "API key validated successfully"
    );

    Ok(Some(api_key))
}

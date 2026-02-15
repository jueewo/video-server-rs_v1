use crate::{generator, ApiKey, ApiKeyResponse, CreateApiKeyRequest, UpdateApiKeyRequest};
use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use tracing::{error, info};

/// Create a new API key for a user
///
/// This generates a secure random key, hashes it, and stores it in the database.
/// The full key is returned in the response - this is the ONLY time it will be visible!
pub async fn create_api_key(
    pool: &SqlitePool,
    user_id: &str,
    request: CreateApiKeyRequest,
) -> Result<ApiKeyResponse> {
    // Validate scopes
    crate::validate_scopes(&request.scopes)?;

    // Generate the API key
    let key = generator::generate_live_key();
    let key_hash = generator::hash_api_key(&key);
    let key_prefix = generator::extract_prefix(&key);

    // Serialize scopes to JSON
    let scopes_json = serde_json::to_string(&request.scopes)?;

    // Insert into database
    let result = sqlx::query(
        r#"
        INSERT INTO user_api_keys (
            user_id, key_hash, key_prefix, name, description, scopes, expires_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&key_hash)
    .bind(&key_prefix)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&scopes_json)
    .bind(&request.expires_at)
    .execute(pool)
    .await?;

    let api_key_id = result.last_insert_rowid() as i32;

    info!(
        event = "api_key_created",
        user_id = %user_id,
        key_id = api_key_id,
        key_prefix = %key_prefix,
        name = %request.name,
        "API key created successfully"
    );

    // Fetch the created key metadata
    let api_key = get_api_key_by_id(pool, api_key_id, user_id).await?
        .ok_or_else(|| anyhow!("Failed to retrieve created API key"))?;

    Ok(ApiKeyResponse { key, api_key })
}

/// Get an API key by its hash (for authentication)
pub async fn get_api_key_by_hash(pool: &SqlitePool, key_hash: &str) -> Result<Option<ApiKey>> {
    let api_key = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM user_api_keys
        WHERE key_hash = ? AND is_active = 1
        "#,
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await?;

    Ok(api_key)
}

/// Get an API key by ID (with ownership check)
pub async fn get_api_key_by_id(
    pool: &SqlitePool,
    key_id: i32,
    user_id: &str,
) -> Result<Option<ApiKey>> {
    let api_key = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM user_api_keys
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(api_key)
}

/// List all API keys for a user
pub async fn list_user_api_keys(pool: &SqlitePool, user_id: &str) -> Result<Vec<ApiKey>> {
    let api_keys = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM user_api_keys
        WHERE user_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(api_keys)
}

/// Revoke (soft delete) an API key
pub async fn revoke_api_key(pool: &SqlitePool, key_id: i32, user_id: &str) -> Result<bool> {
    let result = sqlx::query(
        r#"
        UPDATE user_api_keys
        SET is_active = 0
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            event = "api_key_revoked",
            user_id = %user_id,
            key_id = key_id,
            "API key revoked successfully"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Update API key metadata (name, description, expires_at)
/// Note: Scopes cannot be changed for security reasons
pub async fn update_api_key(
    pool: &SqlitePool,
    key_id: i32,
    user_id: &str,
    request: UpdateApiKeyRequest,
) -> Result<Option<ApiKey>> {
    // Build dynamic update query
    let mut updates = Vec::new();
    let mut query = String::from("UPDATE user_api_keys SET ");

    if request.name.is_some() {
        updates.push("name = ?");
    }
    if request.description.is_some() {
        updates.push("description = ?");
    }
    if request.expires_at.is_some() {
        updates.push("expires_at = ?");
    }

    if updates.is_empty() {
        // Nothing to update, just return current key
        return get_api_key_by_id(pool, key_id, user_id).await;
    }

    query.push_str(&updates.join(", "));
    query.push_str(" WHERE id = ? AND user_id = ?");

    // Build the query with bindings
    let mut sql_query = sqlx::query(&query);

    if let Some(name) = &request.name {
        sql_query = sql_query.bind(name);
    }
    if let Some(description) = &request.description {
        sql_query = sql_query.bind(description);
    }
    if let Some(expires_at) = &request.expires_at {
        sql_query = sql_query.bind(expires_at);
    }

    sql_query = sql_query.bind(key_id).bind(user_id);

    let result = sql_query.execute(pool).await?;

    if result.rows_affected() > 0 {
        info!(
            event = "api_key_updated",
            user_id = %user_id,
            key_id = key_id,
            "API key metadata updated"
        );
        get_api_key_by_id(pool, key_id, user_id).await
    } else {
        Ok(None)
    }
}

/// Update last_used_at timestamp and increment usage_count
pub async fn update_last_used(pool: &SqlitePool, key_id: i32) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE user_api_keys
        SET last_used_at = CURRENT_TIMESTAMP,
            usage_count = usage_count + 1
        WHERE id = ?
        "#,
    )
    .bind(key_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Validate an API key string and return the associated ApiKey if valid
///
/// This function:
/// 1. Hashes the provided key
/// 2. Looks it up in the database
/// 3. Checks if it's active and not expired
/// 4. Updates last_used_at and usage_count if valid
pub async fn validate_api_key(pool: &SqlitePool, key_str: &str) -> Result<Option<ApiKey>> {
    // Hash the key
    let key_hash = generator::hash_api_key(key_str);

    // Look up in database
    let api_key = match get_api_key_by_hash(pool, &key_hash).await? {
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

    // Check if key is valid (active and not expired)
    if !api_key.is_valid() {
        error!(
            event = "api_key_validation_failed",
            reason = if !api_key.is_active { "inactive" } else { "expired" },
            key_id = api_key.id,
            "API key is invalid"
        );
        return Ok(None);
    }

    // Update last_used_at and usage_count
    if let Err(e) = update_last_used(pool, api_key.id).await {
        error!(
            event = "api_key_usage_update_failed",
            error = %e,
            key_id = api_key.id,
            "Failed to update API key usage stats"
        );
        // Continue anyway - authentication is still valid
    }

    info!(
        event = "api_key_validated",
        key_id = api_key.id,
        user_id = %api_key.user_id,
        "API key validated successfully"
    );

    Ok(Some(api_key))
}

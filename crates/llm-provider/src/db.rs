use crate::{crypto, LlmProvider, CreateProviderRequest, UpdateProviderRequest};
use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use tracing::info;

/// Create a new LLM provider for a user.
pub async fn create_provider(
    pool: &SqlitePool,
    user_id: &str,
    request: CreateProviderRequest,
) -> Result<LlmProvider> {
    let encrypted_key = crypto::encrypt_api_key(&request.api_key)?;
    let key_prefix = crypto::extract_key_prefix(&request.api_key);

    // If this provider should be default, unset any existing default first
    if request.is_default {
        sqlx::query("UPDATE user_llm_providers SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    let result = sqlx::query(
        r#"
        INSERT INTO user_llm_providers (
            user_id, name, provider, api_url, api_key_encrypted,
            api_key_prefix, default_model, is_default
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.provider)
    .bind(&request.api_url)
    .bind(&encrypted_key)
    .bind(&key_prefix)
    .bind(&request.default_model)
    .bind(request.is_default)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid() as i32;

    info!(
        event = "llm_provider_created",
        user_id = %user_id,
        provider_id = id,
        name = %request.name,
        provider = %request.provider,
        "LLM provider created"
    );

    get_provider_by_id(pool, id, user_id)
        .await?
        .ok_or_else(|| anyhow!("Failed to retrieve created provider"))
}

/// Get a provider by ID with ownership check.
pub async fn get_provider_by_id(
    pool: &SqlitePool,
    id: i32,
    user_id: &str,
) -> Result<Option<LlmProvider>> {
    let provider = sqlx::query_as::<_, LlmProvider>(
        "SELECT * FROM user_llm_providers WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

/// Get a provider by name for a user.
pub async fn get_provider_by_name(
    pool: &SqlitePool,
    user_id: &str,
    name: &str,
) -> Result<Option<LlmProvider>> {
    let provider = sqlx::query_as::<_, LlmProvider>(
        "SELECT * FROM user_llm_providers WHERE user_id = ? AND name = ?",
    )
    .bind(user_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

/// Get the user's default provider.
pub async fn get_default_provider(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<LlmProvider>> {
    let provider = sqlx::query_as::<_, LlmProvider>(
        "SELECT * FROM user_llm_providers WHERE user_id = ? AND is_default = 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

/// List all providers for a user.
pub async fn list_providers(pool: &SqlitePool, user_id: &str) -> Result<Vec<LlmProvider>> {
    let providers = sqlx::query_as::<_, LlmProvider>(
        "SELECT * FROM user_llm_providers WHERE user_id = ? ORDER BY is_default DESC, name ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(providers)
}

/// Update an existing provider.
pub async fn update_provider(
    pool: &SqlitePool,
    id: i32,
    user_id: &str,
    request: UpdateProviderRequest,
) -> Result<bool> {
    // If new API key provided, encrypt it; otherwise keep existing
    let has_new_key = request.api_key.as_ref().map_or(false, |k| !k.is_empty());

    // If this provider should be default, unset any existing default first
    if request.is_default {
        sqlx::query("UPDATE user_llm_providers SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    let result = if has_new_key {
        let key = request.api_key.as_ref().unwrap();
        let encrypted_key = crypto::encrypt_api_key(key)?;
        let key_prefix = crypto::extract_key_prefix(key);

        sqlx::query(
            r#"
            UPDATE user_llm_providers
            SET name = ?, provider = ?, api_url = ?, api_key_encrypted = ?,
                api_key_prefix = ?, default_model = ?, is_default = ?,
                updated_at = datetime('now')
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(&request.name)
        .bind(&request.provider)
        .bind(&request.api_url)
        .bind(&encrypted_key)
        .bind(&key_prefix)
        .bind(&request.default_model)
        .bind(request.is_default)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            UPDATE user_llm_providers
            SET name = ?, provider = ?, api_url = ?, default_model = ?, is_default = ?,
                updated_at = datetime('now')
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(&request.name)
        .bind(&request.provider)
        .bind(&request.api_url)
        .bind(&request.default_model)
        .bind(request.is_default)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
    };

    if result.rows_affected() > 0 {
        info!(
            event = "llm_provider_updated",
            user_id = %user_id,
            provider_id = id,
            name = %request.name,
            "LLM provider updated"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Delete a provider.
pub async fn delete_provider(pool: &SqlitePool, id: i32, user_id: &str) -> Result<bool> {
    let result = sqlx::query(
        "DELETE FROM user_llm_providers WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            event = "llm_provider_deleted",
            user_id = %user_id,
            provider_id = id,
            "LLM provider deleted"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Set a provider as the default (unsets all others for this user).
pub async fn set_default_provider(pool: &SqlitePool, id: i32, user_id: &str) -> Result<bool> {
    // Unset all defaults
    sqlx::query("UPDATE user_llm_providers SET is_default = 0 WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    // Set the new default
    let result = sqlx::query(
        "UPDATE user_llm_providers SET is_default = 1 WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            event = "llm_provider_set_default",
            user_id = %user_id,
            provider_id = id,
            "LLM provider set as default"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Decrypt the API key for a provider.
pub fn decrypt_provider_key(provider: &LlmProvider) -> Result<String> {
    crypto::decrypt_api_key(&provider.api_key_encrypted)
}

/// Log LLM usage (token counts) for a request.
pub async fn log_usage(
    pool: &SqlitePool,
    user_id: &str,
    provider_id: i32,
    provider_name: &str,
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO llm_usage_log (user_id, provider_id, provider_name, model, input_tokens, output_tokens)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(provider_id)
    .bind(provider_name)
    .bind(model)
    .bind(input_tokens as i64)
    .bind(output_tokens as i64)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get total usage for a user (all time).
pub async fn get_user_usage_summary(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<UsageSummary>> {
    let rows = sqlx::query_as::<_, UsageSummary>(
        r#"
        SELECT provider_name, model,
               SUM(input_tokens) as total_input_tokens,
               SUM(output_tokens) as total_output_tokens,
               COUNT(*) as request_count
        FROM llm_usage_log
        WHERE user_id = ?
        GROUP BY provider_name, model
        ORDER BY request_count DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Usage summary per provider/model combination.
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct UsageSummary {
    pub provider_name: String,
    pub model: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub request_count: i64,
}

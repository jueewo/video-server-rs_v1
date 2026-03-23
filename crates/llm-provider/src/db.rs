use crate::{crypto, CreateProviderRequest, LlmProvider, UpdateProviderRequest};
use anyhow::Result;
use db::llm_providers::{CreateLlmProviderRequest, LlmProviderRepository};
use tracing::info;

// Re-export UsageSummary from the db crate
pub use db::llm_providers::UsageSummary;

/// Create a new LLM provider for a user.
/// Encrypts the plaintext API key before delegating to the repository.
pub async fn create_provider(
    repo: &dyn LlmProviderRepository,
    user_id: &str,
    request: CreateProviderRequest,
) -> Result<LlmProvider> {
    let encrypted_key = crypto::encrypt_api_key(&request.api_key)?;
    let key_prefix = crypto::extract_key_prefix(&request.api_key);

    let db_request = CreateLlmProviderRequest {
        name: request.name.clone(),
        provider: request.provider.clone(),
        api_url: request.api_url,
        api_key_encrypted: encrypted_key,
        api_key_prefix: key_prefix,
        default_model: request.default_model,
        is_default: request.is_default,
    };

    let provider = repo.create_provider(user_id, &db_request).await?;

    info!(
        event = "llm_provider_created",
        user_id = %user_id,
        provider_id = provider.id,
        name = %request.name,
        provider_type = %request.provider,
        "LLM provider created"
    );

    Ok(provider)
}

/// Get a provider by ID with ownership check.
pub async fn get_provider_by_id(
    repo: &dyn LlmProviderRepository,
    id: i32,
    user_id: &str,
) -> Result<Option<LlmProvider>> {
    Ok(repo.get_provider_by_id(id, user_id).await?)
}

/// Get a provider by name for a user.
pub async fn get_provider_by_name(
    repo: &dyn LlmProviderRepository,
    user_id: &str,
    name: &str,
) -> Result<Option<LlmProvider>> {
    Ok(repo.get_provider_by_name(user_id, name).await?)
}

/// Get the user's default provider.
pub async fn get_default_provider(
    repo: &dyn LlmProviderRepository,
    user_id: &str,
) -> Result<Option<LlmProvider>> {
    Ok(repo.get_default_provider(user_id).await?)
}

/// List all providers for a user.
pub async fn list_providers(
    repo: &dyn LlmProviderRepository,
    user_id: &str,
) -> Result<Vec<LlmProvider>> {
    Ok(repo.list_providers(user_id).await?)
}

/// Update an existing provider.
/// Encrypts the new API key if provided before delegating to the repository.
pub async fn update_provider(
    repo: &dyn LlmProviderRepository,
    id: i32,
    user_id: &str,
    request: UpdateProviderRequest,
) -> Result<bool> {
    let has_new_key = request.api_key.as_ref().map_or(false, |k| !k.is_empty());

    let (new_key_encrypted, new_key_prefix) = if has_new_key {
        let key = request.api_key.as_ref().unwrap();
        let encrypted = crypto::encrypt_api_key(key)?;
        let prefix = crypto::extract_key_prefix(key);
        (Some(encrypted), Some(prefix))
    } else {
        (None, None)
    };

    let updated = repo
        .update_provider(
            id,
            user_id,
            &request.name,
            &request.provider,
            &request.api_url,
            &request.default_model,
            request.is_default,
            new_key_encrypted.as_deref(),
            new_key_prefix.as_deref(),
        )
        .await?;

    if updated {
        info!(
            event = "llm_provider_updated",
            user_id = %user_id,
            provider_id = id,
            name = %request.name,
            "LLM provider updated"
        );
    }

    Ok(updated)
}

/// Delete a provider.
pub async fn delete_provider(
    repo: &dyn LlmProviderRepository,
    id: i32,
    user_id: &str,
) -> Result<bool> {
    let deleted = repo.delete_provider(id, user_id).await?;

    if deleted {
        info!(
            event = "llm_provider_deleted",
            user_id = %user_id,
            provider_id = id,
            "LLM provider deleted"
        );
    }

    Ok(deleted)
}

/// Set a provider as the default (unsets all others for this user).
pub async fn set_default_provider(
    repo: &dyn LlmProviderRepository,
    id: i32,
    user_id: &str,
) -> Result<bool> {
    let updated = repo.set_default_provider(id, user_id).await?;

    if updated {
        info!(
            event = "llm_provider_set_default",
            user_id = %user_id,
            provider_id = id,
            "LLM provider set as default"
        );
    }

    Ok(updated)
}

/// Decrypt the API key for a provider.
pub fn decrypt_provider_key(provider: &LlmProvider) -> Result<String> {
    crypto::decrypt_api_key(&provider.api_key_encrypted)
}

/// Log LLM usage (token counts) for a request.
pub async fn log_usage(
    repo: &dyn LlmProviderRepository,
    user_id: &str,
    _provider_id: i32,
    provider_name: &str,
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
) -> Result<()> {
    repo.log_usage(
        user_id,
        provider_name,
        model,
        input_tokens as i64,
        output_tokens as i64,
    )
    .await?;

    Ok(())
}

/// Get total usage for a user (all time).
pub async fn get_user_usage_summary(
    repo: &dyn LlmProviderRepository,
    user_id: &str,
) -> Result<Vec<UsageSummary>> {
    Ok(repo.get_user_usage_summary(user_id).await?)
}

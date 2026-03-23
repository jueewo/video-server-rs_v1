use crate::{crypto, CreateGitProviderInput, GitProvider, UpdateGitProviderInput};
use anyhow::Result;
use db::git_providers::GitProviderRepository;
use tracing::info;

pub async fn create_provider(
    repo: &dyn GitProviderRepository,
    user_id: &str,
    request: CreateGitProviderInput,
) -> Result<GitProvider> {
    let encrypted_token = crypto::encrypt_token(&request.token)?;
    let token_prefix = crypto::extract_token_prefix(&request.token);

    let db_request = db::git_providers::CreateGitProviderRequest {
        name: request.name.clone(),
        provider_type: request.provider_type.clone(),
        base_url: request.base_url.clone(),
        token_encrypted: encrypted_token,
        token_prefix,
        is_default: request.is_default,
    };

    let provider = repo.create_provider(user_id, &db_request).await?;

    info!(
        event = "git_provider_created",
        user_id = %user_id,
        provider_id = provider.id,
        name = %request.name,
        provider_type = %request.provider_type,
        "Git provider created"
    );

    Ok(provider)
}

pub async fn get_provider_by_id(
    repo: &dyn GitProviderRepository,
    id: i32,
    user_id: &str,
) -> Result<Option<GitProvider>> {
    Ok(repo.get_provider_by_id(id, user_id).await?)
}

pub async fn get_provider_by_name(
    repo: &dyn GitProviderRepository,
    user_id: &str,
    name: &str,
) -> Result<Option<GitProvider>> {
    Ok(repo.get_provider_by_name(user_id, name).await?)
}

pub async fn get_default_provider(
    repo: &dyn GitProviderRepository,
    user_id: &str,
) -> Result<Option<GitProvider>> {
    Ok(repo.get_default_provider(user_id).await?)
}

pub async fn list_providers(
    repo: &dyn GitProviderRepository,
    user_id: &str,
) -> Result<Vec<GitProvider>> {
    Ok(repo.list_providers(user_id).await?)
}

pub async fn update_provider(
    repo: &dyn GitProviderRepository,
    id: i32,
    user_id: &str,
    request: UpdateGitProviderInput,
) -> Result<bool> {
    let (new_token_encrypted, new_token_prefix) =
        if let Some(ref token) = request.token.as_ref().filter(|t| !t.is_empty()) {
            let encrypted = crypto::encrypt_token(token)?;
            let prefix = crypto::extract_token_prefix(token);
            (Some(encrypted), Some(prefix))
        } else {
            (None, None)
        };

    let db_request = db::git_providers::UpdateGitProviderRequest {
        name: request.name.clone(),
        provider_type: request.provider_type.clone(),
        base_url: request.base_url.clone(),
        is_default: request.is_default,
        new_token_encrypted,
        new_token_prefix,
    };

    let updated = repo.update_provider(id, user_id, &db_request).await?;

    if updated {
        info!(
            event = "git_provider_updated",
            user_id = %user_id,
            provider_id = id,
            name = %request.name,
            "Git provider updated"
        );
    }

    Ok(updated)
}

pub async fn delete_provider(
    repo: &dyn GitProviderRepository,
    id: i32,
    user_id: &str,
) -> Result<bool> {
    let deleted = repo.delete_provider(id, user_id).await?;

    if deleted {
        info!(
            event = "git_provider_deleted",
            user_id = %user_id,
            provider_id = id,
            "Git provider deleted"
        );
    }

    Ok(deleted)
}

pub async fn set_default_provider(
    repo: &dyn GitProviderRepository,
    id: i32,
    user_id: &str,
) -> Result<bool> {
    let updated = repo.set_default_provider(id, user_id).await?;

    if updated {
        info!(
            event = "git_provider_set_default",
            user_id = %user_id,
            provider_id = id,
            "Git provider set as default"
        );
    }

    Ok(updated)
}

pub fn decrypt_provider_token(provider: &GitProvider) -> Result<String> {
    crypto::decrypt_token(&provider.token_encrypted)
}

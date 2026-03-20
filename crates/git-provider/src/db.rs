use crate::{crypto, CreateGitProviderRequest, GitProvider, UpdateGitProviderRequest};
use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use tracing::info;

pub async fn create_provider(
    pool: &SqlitePool,
    user_id: &str,
    request: CreateGitProviderRequest,
) -> Result<GitProvider> {
    let encrypted_token = crypto::encrypt_token(&request.token)?;
    let token_prefix = crypto::extract_token_prefix(&request.token);

    if request.is_default {
        sqlx::query("UPDATE user_git_providers SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    let result = sqlx::query(
        r#"
        INSERT INTO user_git_providers (
            user_id, name, provider_type, base_url,
            token_encrypted, token_prefix, is_default
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.provider_type)
    .bind(&request.base_url)
    .bind(&encrypted_token)
    .bind(&token_prefix)
    .bind(request.is_default)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid() as i32;

    info!(
        event = "git_provider_created",
        user_id = %user_id,
        provider_id = id,
        name = %request.name,
        provider_type = %request.provider_type,
        "Git provider created"
    );

    get_provider_by_id(pool, id, user_id)
        .await?
        .ok_or_else(|| anyhow!("Failed to retrieve created provider"))
}

pub async fn get_provider_by_id(
    pool: &SqlitePool,
    id: i32,
    user_id: &str,
) -> Result<Option<GitProvider>> {
    let provider = sqlx::query_as::<_, GitProvider>(
        "SELECT * FROM user_git_providers WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

pub async fn get_provider_by_name(
    pool: &SqlitePool,
    user_id: &str,
    name: &str,
) -> Result<Option<GitProvider>> {
    let provider = sqlx::query_as::<_, GitProvider>(
        "SELECT * FROM user_git_providers WHERE user_id = ? AND name = ?",
    )
    .bind(user_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

pub async fn get_default_provider(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<GitProvider>> {
    let provider = sqlx::query_as::<_, GitProvider>(
        "SELECT * FROM user_git_providers WHERE user_id = ? AND is_default = 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

pub async fn list_providers(pool: &SqlitePool, user_id: &str) -> Result<Vec<GitProvider>> {
    let providers = sqlx::query_as::<_, GitProvider>(
        "SELECT * FROM user_git_providers WHERE user_id = ? ORDER BY is_default DESC, name ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(providers)
}

pub async fn update_provider(
    pool: &SqlitePool,
    id: i32,
    user_id: &str,
    request: UpdateGitProviderRequest,
) -> Result<bool> {
    let has_new_token = request.token.as_ref().map_or(false, |t| !t.is_empty());

    if request.is_default {
        sqlx::query("UPDATE user_git_providers SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    let result = if has_new_token {
        let token = request.token.as_ref().unwrap();
        let encrypted = crypto::encrypt_token(token)?;
        let prefix = crypto::extract_token_prefix(token);

        sqlx::query(
            r#"
            UPDATE user_git_providers
            SET name = ?, provider_type = ?, base_url = ?, token_encrypted = ?,
                token_prefix = ?, is_default = ?, updated_at = datetime('now')
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(&request.name)
        .bind(&request.provider_type)
        .bind(&request.base_url)
        .bind(&encrypted)
        .bind(&prefix)
        .bind(request.is_default)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            UPDATE user_git_providers
            SET name = ?, provider_type = ?, base_url = ?, is_default = ?,
                updated_at = datetime('now')
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(&request.name)
        .bind(&request.provider_type)
        .bind(&request.base_url)
        .bind(request.is_default)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
    };

    if result.rows_affected() > 0 {
        info!(
            event = "git_provider_updated",
            user_id = %user_id,
            provider_id = id,
            name = %request.name,
            "Git provider updated"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn delete_provider(pool: &SqlitePool, id: i32, user_id: &str) -> Result<bool> {
    let result = sqlx::query(
        "DELETE FROM user_git_providers WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            event = "git_provider_deleted",
            user_id = %user_id,
            provider_id = id,
            "Git provider deleted"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn set_default_provider(pool: &SqlitePool, id: i32, user_id: &str) -> Result<bool> {
    sqlx::query("UPDATE user_git_providers SET is_default = 0 WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    let result = sqlx::query(
        "UPDATE user_git_providers SET is_default = 1 WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            event = "git_provider_set_default",
            user_id = %user_id,
            provider_id = id,
            "Git provider set as default"
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn decrypt_provider_token(provider: &GitProvider) -> Result<String> {
    crypto::decrypt_token(&provider.token_encrypted)
}

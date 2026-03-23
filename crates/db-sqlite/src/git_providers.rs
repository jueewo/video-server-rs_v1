//! SQLite implementation of [`db::git_providers::GitProviderRepository`].

use db::git_providers::{
    CreateGitProviderRequest, GitProvider, GitProviderRepository, UpdateGitProviderRequest,
};
use db::DbError;

use crate::SqliteDatabase;

// ============================================================================
// Internal row type
// ============================================================================

#[derive(sqlx::FromRow)]
struct GitProviderRow {
    id: i32,
    user_id: String,
    name: String,
    provider_type: String,
    base_url: String,
    token_encrypted: String,
    token_prefix: String,
    is_default: bool,
    created_at: String,
    updated_at: String,
}

impl From<GitProviderRow> for GitProvider {
    fn from(r: GitProviderRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            name: r.name,
            provider_type: r.provider_type,
            base_url: r.base_url,
            token_encrypted: r.token_encrypted,
            token_prefix: r.token_prefix,
            is_default: r.is_default,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

fn map_err(e: sqlx::Error) -> DbError {
    match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            DbError::UniqueViolation(db_err.message().to_string())
        }
        _ => DbError::Internal(e.to_string()),
    }
}

// ============================================================================
// Trait implementation
// ============================================================================

#[async_trait::async_trait]
impl GitProviderRepository for SqliteDatabase {
    async fn create_provider(
        &self,
        user_id: &str,
        req: &CreateGitProviderRequest,
    ) -> Result<GitProvider, DbError> {
        if req.is_default {
            sqlx::query("UPDATE user_git_providers SET is_default = 0 WHERE user_id = ?")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(map_err)?;
        }

        let result = sqlx::query(
            "INSERT INTO user_git_providers \
             (user_id, name, provider_type, base_url, token_encrypted, token_prefix, is_default) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(&req.name)
        .bind(&req.provider_type)
        .bind(&req.base_url)
        .bind(&req.token_encrypted)
        .bind(&req.token_prefix)
        .bind(req.is_default)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        let id = result.last_insert_rowid() as i32;

        self.get_provider_by_id(id, user_id)
            .await?
            .ok_or_else(|| DbError::internal("Failed to retrieve created provider"))
    }

    async fn get_provider_by_id(&self, id: i32, user_id: &str) -> Result<Option<GitProvider>, DbError> {
        let row: Option<GitProviderRow> = sqlx::query_as(
            "SELECT * FROM user_git_providers WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn get_provider_by_name(&self, user_id: &str, name: &str) -> Result<Option<GitProvider>, DbError> {
        let row: Option<GitProviderRow> = sqlx::query_as(
            "SELECT * FROM user_git_providers WHERE user_id = ? AND name = ?",
        )
        .bind(user_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn get_default_provider(&self, user_id: &str) -> Result<Option<GitProvider>, DbError> {
        let row: Option<GitProviderRow> = sqlx::query_as(
            "SELECT * FROM user_git_providers WHERE user_id = ? AND is_default = 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn list_providers(&self, user_id: &str) -> Result<Vec<GitProvider>, DbError> {
        let rows: Vec<GitProviderRow> = sqlx::query_as(
            "SELECT * FROM user_git_providers WHERE user_id = ? ORDER BY is_default DESC, name ASC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_provider(
        &self,
        id: i32,
        user_id: &str,
        req: &UpdateGitProviderRequest,
    ) -> Result<bool, DbError> {
        if req.is_default {
            sqlx::query("UPDATE user_git_providers SET is_default = 0 WHERE user_id = ?")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(map_err)?;
        }

        let result = if let (Some(encrypted), Some(prefix)) = (&req.new_token_encrypted, &req.new_token_prefix) {
            sqlx::query(
                "UPDATE user_git_providers SET \
                 name = ?, provider_type = ?, base_url = ?, token_encrypted = ?, \
                 token_prefix = ?, is_default = ?, updated_at = datetime('now') \
                 WHERE id = ? AND user_id = ?",
            )
            .bind(&req.name)
            .bind(&req.provider_type)
            .bind(&req.base_url)
            .bind(encrypted)
            .bind(prefix)
            .bind(req.is_default)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?
        } else {
            sqlx::query(
                "UPDATE user_git_providers SET \
                 name = ?, provider_type = ?, base_url = ?, is_default = ?, \
                 updated_at = datetime('now') \
                 WHERE id = ? AND user_id = ?",
            )
            .bind(&req.name)
            .bind(&req.provider_type)
            .bind(&req.base_url)
            .bind(req.is_default)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?
        };

        Ok(result.rows_affected() > 0)
    }

    async fn delete_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError> {
        let result = sqlx::query(
            "DELETE FROM user_git_providers WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn set_default_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError> {
        sqlx::query("UPDATE user_git_providers SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;

        let result = sqlx::query(
            "UPDATE user_git_providers SET is_default = 1, updated_at = datetime('now') WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }
}

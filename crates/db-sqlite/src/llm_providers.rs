//! SQLite implementation of [`db::llm_providers::LlmProviderRepository`].

use db::llm_providers::{
    CreateLlmProviderRequest, LlmProvider, LlmProviderRepository, UsageSummary,
};
use db::DbError;

use crate::SqliteDatabase;

// ============================================================================
// Internal row types
// ============================================================================

#[derive(sqlx::FromRow)]
struct LlmProviderRow {
    id: i32,
    user_id: String,
    name: String,
    provider: String,
    api_url: String,
    api_key_encrypted: String,
    api_key_prefix: String,
    default_model: String,
    is_default: bool,
    created_at: String,
    updated_at: String,
}

impl From<LlmProviderRow> for LlmProvider {
    fn from(r: LlmProviderRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            name: r.name,
            provider: r.provider,
            api_url: r.api_url,
            api_key_encrypted: r.api_key_encrypted,
            api_key_prefix: r.api_key_prefix,
            default_model: r.default_model,
            is_default: r.is_default,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct UsageSummaryRow {
    provider_name: String,
    model: String,
    total_input_tokens: i64,
    total_output_tokens: i64,
    request_count: i64,
}

impl From<UsageSummaryRow> for UsageSummary {
    fn from(r: UsageSummaryRow) -> Self {
        Self {
            provider_name: r.provider_name,
            model: r.model,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            request_count: r.request_count,
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
impl LlmProviderRepository for SqliteDatabase {
    async fn create_provider(
        &self,
        user_id: &str,
        req: &CreateLlmProviderRequest,
    ) -> Result<LlmProvider, DbError> {
        if req.is_default {
            sqlx::query("UPDATE user_llm_providers SET is_default = 0 WHERE user_id = ?")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(map_err)?;
        }

        let result = sqlx::query(
            "INSERT INTO user_llm_providers \
             (user_id, name, provider, api_url, api_key_encrypted, api_key_prefix, default_model, is_default) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(&req.name)
        .bind(&req.provider)
        .bind(&req.api_url)
        .bind(&req.api_key_encrypted)
        .bind(&req.api_key_prefix)
        .bind(&req.default_model)
        .bind(req.is_default)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        let id = result.last_insert_rowid() as i32;

        self.get_provider_by_id(id, user_id)
            .await?
            .ok_or_else(|| DbError::internal("Failed to retrieve created provider"))
    }

    async fn get_provider_by_id(&self, id: i32, user_id: &str) -> Result<Option<LlmProvider>, DbError> {
        let row: Option<LlmProviderRow> = sqlx::query_as(
            "SELECT * FROM user_llm_providers WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn get_provider_by_name(&self, user_id: &str, name: &str) -> Result<Option<LlmProvider>, DbError> {
        let row: Option<LlmProviderRow> = sqlx::query_as(
            "SELECT * FROM user_llm_providers WHERE user_id = ? AND name = ?",
        )
        .bind(user_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn get_default_provider(&self, user_id: &str) -> Result<Option<LlmProvider>, DbError> {
        let row: Option<LlmProviderRow> = sqlx::query_as(
            "SELECT * FROM user_llm_providers WHERE user_id = ? AND is_default = 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn list_providers(&self, user_id: &str) -> Result<Vec<LlmProvider>, DbError> {
        let rows: Vec<LlmProviderRow> = sqlx::query_as(
            "SELECT * FROM user_llm_providers WHERE user_id = ? ORDER BY is_default DESC, name ASC",
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
        name: &str,
        provider: &str,
        api_url: &str,
        default_model: &str,
        is_default: bool,
        new_key_encrypted: Option<&str>,
        new_key_prefix: Option<&str>,
    ) -> Result<bool, DbError> {
        if is_default {
            sqlx::query("UPDATE user_llm_providers SET is_default = 0 WHERE user_id = ?")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(map_err)?;
        }

        let result = if let (Some(encrypted), Some(prefix)) = (new_key_encrypted, new_key_prefix) {
            sqlx::query(
                "UPDATE user_llm_providers SET \
                 name = ?, provider = ?, api_url = ?, api_key_encrypted = ?, api_key_prefix = ?, \
                 default_model = ?, is_default = ?, updated_at = datetime('now') \
                 WHERE id = ? AND user_id = ?",
            )
            .bind(name)
            .bind(provider)
            .bind(api_url)
            .bind(encrypted)
            .bind(prefix)
            .bind(default_model)
            .bind(is_default)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?
        } else {
            sqlx::query(
                "UPDATE user_llm_providers SET \
                 name = ?, provider = ?, api_url = ?, default_model = ?, is_default = ?, \
                 updated_at = datetime('now') \
                 WHERE id = ? AND user_id = ?",
            )
            .bind(name)
            .bind(provider)
            .bind(api_url)
            .bind(default_model)
            .bind(is_default)
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
            "DELETE FROM user_llm_providers WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn set_default_provider(&self, id: i32, user_id: &str) -> Result<bool, DbError> {
        sqlx::query("UPDATE user_llm_providers SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;

        let result = sqlx::query(
            "UPDATE user_llm_providers SET is_default = 1, updated_at = datetime('now') WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn log_usage(
        &self,
        user_id: &str,
        provider_name: &str,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO llm_usage_log (user_id, provider_name, model, input_tokens, output_tokens) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(provider_name)
        .bind(model)
        .bind(input_tokens)
        .bind(output_tokens)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }

    async fn get_user_usage_summary(&self, user_id: &str) -> Result<Vec<UsageSummary>, DbError> {
        let rows: Vec<UsageSummaryRow> = sqlx::query_as(
            "SELECT provider_name, model, \
             SUM(input_tokens) as total_input_tokens, \
             SUM(output_tokens) as total_output_tokens, \
             COUNT(*) as request_count \
             FROM llm_usage_log WHERE user_id = ? \
             GROUP BY provider_name, model \
             ORDER BY request_count DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

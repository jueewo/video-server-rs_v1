//! SQLite implementation of [`db::api_keys::ApiKeyRepository`].

use db::api_keys::{ApiKey, ApiKeyRepository, CreateApiKeyRequest, UpdateApiKeyRequest};
use db::DbError;

use crate::SqliteDatabase;

// ============================================================================
// Internal row type
// ============================================================================

#[derive(sqlx::FromRow)]
struct ApiKeyRow {
    id: i32,
    user_id: String,
    key_prefix: String,
    name: String,
    description: Option<String>,
    scopes: String,
    last_used_at: Option<String>,
    usage_count: i32,
    expires_at: Option<String>,
    created_at: String,
    is_active: bool,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(r: ApiKeyRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            key_prefix: r.key_prefix,
            name: r.name,
            description: r.description,
            scopes: r.scopes,
            last_used_at: r.last_used_at,
            usage_count: r.usage_count,
            expires_at: r.expires_at,
            created_at: r.created_at,
            is_active: r.is_active,
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
impl ApiKeyRepository for SqliteDatabase {
    async fn create_api_key(
        &self,
        user_id: &str,
        _key: &str,
        key_hash: &str,
        key_prefix: &str,
        request: &CreateApiKeyRequest,
    ) -> Result<ApiKey, DbError> {
        let scopes_json = serde_json::to_string(&request.scopes).unwrap_or_else(|_| "[]".into());

        let result = sqlx::query(
            "INSERT INTO user_api_keys (user_id, key_hash, key_prefix, name, description, scopes, expires_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(key_hash)
        .bind(key_prefix)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&scopes_json)
        .bind(&request.expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        let id = result.last_insert_rowid() as i32;

        self.get_api_key_by_id(id, user_id)
            .await?
            .ok_or_else(|| DbError::internal("Failed to retrieve created API key"))
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DbError> {
        let row: Option<ApiKeyRow> = sqlx::query_as(
            "SELECT * FROM user_api_keys WHERE key_hash = ? AND is_active = 1",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn get_api_key_by_id(&self, key_id: i32, user_id: &str) -> Result<Option<ApiKey>, DbError> {
        let row: Option<ApiKeyRow> = sqlx::query_as(
            "SELECT * FROM user_api_keys WHERE id = ? AND user_id = ?",
        )
        .bind(key_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }

    async fn list_user_api_keys(&self, user_id: &str) -> Result<Vec<ApiKey>, DbError> {
        let rows: Vec<ApiKeyRow> = sqlx::query_as(
            "SELECT * FROM user_api_keys WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn revoke_api_key(&self, key_id: i32, user_id: &str) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE user_api_keys SET is_active = 0 WHERE id = ? AND user_id = ?",
        )
        .bind(key_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_api_key(
        &self,
        key_id: i32,
        user_id: &str,
        request: &UpdateApiKeyRequest,
    ) -> Result<Option<ApiKey>, DbError> {
        let mut updates = Vec::new();

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
            return self.get_api_key_by_id(key_id, user_id).await;
        }

        let query_str = format!(
            "UPDATE user_api_keys SET {} WHERE id = ? AND user_id = ?",
            updates.join(", ")
        );

        let mut sql_query = sqlx::query(&query_str);

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

        let result = sql_query.execute(&self.pool).await.map_err(map_err)?;

        if result.rows_affected() > 0 {
            self.get_api_key_by_id(key_id, user_id).await
        } else {
            Ok(None)
        }
    }

    async fn update_last_used(&self, key_id: i32) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE user_api_keys SET last_used_at = CURRENT_TIMESTAMP, usage_count = usage_count + 1 WHERE id = ?",
        )
        .bind(key_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }
}

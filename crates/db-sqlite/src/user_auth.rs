use crate::SqliteDatabase;
use db::user_auth::{UpsertUserRequest, UserAuthRepository, UserLoginInfo};
use db::DbError;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

#[async_trait::async_trait]
impl UserAuthRepository for SqliteDatabase {
    async fn upsert_user(&self, req: &UpsertUserRequest<'_>) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, name, avatar_url, provider, last_login_at)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                name = excluded.name,
                avatar_url = excluded.avatar_url,
                last_login_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(req.id)
        .bind(req.email)
        .bind(req.name)
        .bind(req.avatar_url)
        .bind(req.provider)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn get_user_login_info(&self, user_id: &str) -> Result<Option<UserLoginInfo>, DbError> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT provider, last_login_at FROM users WHERE id = ? LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|(provider, last_login_at)| UserLoginInfo {
            provider,
            last_login_at,
        }))
    }

    async fn get_user_tenant_id(&self, user_id: &str) -> Result<Option<String>, DbError> {
        let tenant_id: Option<String> =
            sqlx::query_scalar("SELECT tenant_id FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(tenant_id)
    }

    async fn set_user_tenant(&self, user_id: &str, tenant_id: &str) -> Result<(), DbError> {
        sqlx::query("UPDATE users SET tenant_id = ? WHERE id = ?")
            .bind(tenant_id)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn get_tenant_invitation_by_email(&self, email: &str) -> Result<Option<String>, DbError> {
        let tenant_id: Option<String> = sqlx::query_scalar(
            "SELECT tenant_id FROM tenant_invitations WHERE email = ? LIMIT 1",
        )
        .bind(email)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(tenant_id)
    }

    async fn delete_tenant_invitation(&self, email: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM tenant_invitations WHERE email = ?")
            .bind(email)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn get_tenant_branding_json(&self, tenant_id: &str) -> Result<Option<String>, DbError> {
        let branding: Option<Option<String>> =
            sqlx::query_scalar("SELECT branding FROM tenants WHERE id = ?")
                .bind(tenant_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        // flatten: column can be NULL, and the row might not exist
        Ok(branding.flatten())
    }

    async fn get_user_email(&self, user_id: &str) -> Result<Option<String>, DbError> {
        sqlx::query_scalar("SELECT email FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)
    }

    async fn get_user_name(&self, user_id: &str) -> Result<Option<Option<String>>, DbError> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT name FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row.map(|(name,)| name))
    }
}

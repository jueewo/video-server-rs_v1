use crate::SqliteDatabase;
use db::access_codes::{AccessCode, AccessCodePermission, AccessCodeRepository};
use db::DbError;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

fn row_to_access_code(row: (i32, String, Option<String>, Option<String>, String, String, Option<String>, i32, i64)) -> AccessCode {
    AccessCode {
        id: row.0,
        code: row.1,
        description: row.2,
        expires_at: row.3,
        created_at: row.4,
        created_by: row.5,
        vault_id: row.6,
        is_active: row.7 != 0,
        current_downloads: row.8,
    }
}

#[async_trait::async_trait]
impl AccessCodeRepository for SqliteDatabase {
    async fn code_exists(&self, code: &str) -> Result<bool, DbError> {
        let exists: Option<i32> =
            sqlx::query_scalar("SELECT id FROM access_codes WHERE code = ?")
                .bind(code)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(exists.is_some())
    }

    async fn create_code(
        &self,
        code: &str,
        description: Option<&str>,
        expires_at: Option<&str>,
        created_by: &str,
        vault_id: Option<&str>,
    ) -> Result<i32, DbError> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO access_codes (code, description, expires_at, created_by, vault_id) \
             VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(code)
        .bind(description)
        .bind(expires_at)
        .bind(created_by)
        .bind(vault_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;
        Ok(id)
    }

    async fn get_code_by_code_and_user(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<Option<AccessCode>, DbError> {
        let row: Option<(i32, String, Option<String>, Option<String>, String, String, Option<String>, i32, i64)> = sqlx::query_as(
            "SELECT id, code, description, expires_at, created_at, created_by, vault_id, is_active, current_downloads \
             FROM access_codes WHERE code = ? AND created_by = ?",
        )
        .bind(code)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(row_to_access_code))
    }

    async fn get_active_code(&self, code: &str) -> Result<Option<AccessCode>, DbError> {
        let row: Option<(i32, String, Option<String>, Option<String>, String, String, Option<String>, i32, i64)> = sqlx::query_as(
            "SELECT id, code, description, expires_at, created_at, created_by, vault_id, is_active, current_downloads \
             FROM access_codes WHERE code = ? AND is_active = 1",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(row_to_access_code))
    }

    async fn list_user_codes(&self, user_id: &str) -> Result<Vec<AccessCode>, DbError> {
        let rows: Vec<(i32, String, Option<String>, Option<String>, String, String, Option<String>, i32, i64)> = sqlx::query_as(
            "SELECT id, code, description, expires_at, created_at, created_by, vault_id, is_active, current_downloads \
             FROM access_codes WHERE created_by = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows.into_iter().map(row_to_access_code).collect())
    }

    async fn delete_code(&self, code: &str, user_id: &str) -> Result<bool, DbError> {
        let result =
            sqlx::query("DELETE FROM access_codes WHERE code = ? AND created_by = ?")
                .bind(code)
                .bind(user_id)
                .execute(self.pool())
                .await
                .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_code_id_for_user(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<Option<i32>, DbError> {
        let id: Option<i32> =
            sqlx::query_scalar("SELECT id FROM access_codes WHERE code = ? AND created_by = ?")
                .bind(code)
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(id)
    }

    // ── Permissions ─────────────────────────────────────────────────

    async fn add_permission(
        &self,
        code_id: i32,
        media_type: &str,
        media_slug: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT OR IGNORE INTO access_code_permissions (access_code_id, media_type, media_slug) \
             VALUES (?, ?, ?)",
        )
        .bind(code_id)
        .bind(media_type)
        .bind(media_slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn remove_permission(&self, code_id: i32, media_slug: &str) -> Result<bool, DbError> {
        let result = sqlx::query(
            "DELETE FROM access_code_permissions WHERE access_code_id = ? AND media_slug = ?",
        )
        .bind(code_id)
        .bind(media_slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_permissions(&self, code_id: i32) -> Result<Vec<AccessCodePermission>, DbError> {
        let rows: Vec<(i32, String, String)> = sqlx::query_as(
            "SELECT access_code_id, media_type, media_slug \
             FROM access_code_permissions WHERE access_code_id = ?",
        )
        .bind(code_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|(access_code_id, media_type, media_slug)| AccessCodePermission {
                access_code_id,
                media_type,
                media_slug,
            })
            .collect())
    }
}

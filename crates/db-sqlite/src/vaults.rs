use crate::SqliteDatabase;
use db::vaults::{InsertVaultRequest, StorageVault, VaultRepository};
use db::DbError;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

#[async_trait::async_trait]
impl VaultRepository for SqliteDatabase {
    async fn get_default_vault_id(&self, user_id: &str) -> Result<Option<String>, DbError> {
        sqlx::query_scalar(
            "SELECT vault_id FROM storage_vaults WHERE user_id = ? AND is_default = 1",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)
    }

    async fn vault_id_exists(&self, vault_id: &str) -> Result<bool, DbError> {
        let exists: Option<i32> =
            sqlx::query_scalar("SELECT 1 FROM storage_vaults WHERE vault_id = ?")
                .bind(vault_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(exists.is_some())
    }

    async fn count_user_vaults(&self, user_id: &str) -> Result<i64, DbError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM storage_vaults WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(self.pool())
                .await
                .map_err(map_err)?;
        Ok(count)
    }

    async fn insert_vault(&self, req: &InsertVaultRequest<'_>) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO storage_vaults (vault_id, user_id, vault_name, is_default, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(req.vault_id)
        .bind(req.user_id)
        .bind(req.vault_name)
        .bind(if req.is_default { 1 } else { 0 })
        .bind(req.created_at)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn list_user_vaults(&self, user_id: &str) -> Result<Vec<StorageVault>, DbError> {
        let rows: Vec<(String, String, String, i32, String)> = sqlx::query_as(
            "SELECT vault_id, user_id, vault_name, is_default, created_at \
             FROM storage_vaults WHERE user_id = ? \
             ORDER BY is_default DESC, created_at ASC",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(vault_id, user_id, vault_name, is_default, created_at)| StorageVault {
                vault_id,
                user_id,
                vault_name,
                is_default: is_default != 0,
                created_at,
            })
            .collect())
    }

    async fn count_vault_media(&self, vault_id: &str) -> Result<i64, DbError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM media_items WHERE vault_id = ?")
                .bind(vault_id)
                .fetch_one(self.pool())
                .await
                .map_err(map_err)?;
        Ok(count)
    }

    async fn get_vault_owner(&self, vault_id: &str) -> Result<Option<String>, DbError> {
        let owner: Option<String> =
            sqlx::query_scalar("SELECT user_id FROM storage_vaults WHERE vault_id = ?")
                .bind(vault_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(owner)
    }

    async fn update_vault_name(&self, vault_id: &str, name: &str) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE storage_vaults SET vault_name = ?, updated_at = datetime('now') WHERE vault_id = ?",
        )
        .bind(name)
        .bind(vault_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn set_default_vault(&self, user_id: &str, vault_id: &str) -> Result<(), DbError> {
        let mut tx = self.pool().begin().await.map_err(map_err)?;

        sqlx::query("UPDATE storage_vaults SET is_default = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

        sqlx::query(
            "UPDATE storage_vaults SET is_default = 1, updated_at = datetime('now') WHERE vault_id = ?",
        )
        .bind(vault_id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

        tx.commit().await.map_err(map_err)?;
        Ok(())
    }

    async fn delete_vault(&self, vault_id: &str, user_id: &str) -> Result<bool, DbError> {
        let result =
            sqlx::query("DELETE FROM storage_vaults WHERE vault_id = ? AND user_id = ?")
                .bind(vault_id)
                .bind(user_id)
                .execute(self.pool())
                .await
                .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }
}

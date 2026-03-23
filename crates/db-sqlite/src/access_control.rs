//! SQLite implementation of access control repository traits.

use crate::SqliteDatabase;
use db::access_control::*;
use db::DbError;

fn map_err(e: sqlx::Error) -> DbError {
    DbError::Internal(e.to_string())
}

// ── AccessControlRepository ────────────────────────────────────────

#[async_trait::async_trait]
impl AccessControlRepository for SqliteDatabase {
    async fn is_resource_public(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<bool>, DbError> {
        let result: Option<i32> = match resource_type {
            "video" => {
                sqlx::query_scalar(
                    "SELECT is_public FROM media_items WHERE id = ? AND media_type = 'video'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?
            }
            "image" => {
                sqlx::query_scalar(
                    "SELECT is_public FROM media_items WHERE id = ? AND media_type = 'image'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?
            }
            "document" => {
                sqlx::query_scalar(
                    "SELECT is_public FROM media_items WHERE id = ? AND media_type = 'document'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?
            }
            "folder" => {
                sqlx::query_scalar("SELECT is_public FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(self.pool())
                    .await
                    .map_err(map_err)?
            }
            _ => return Ok(None),
        };
        Ok(result.map(|v| v == 1))
    }

    async fn get_resource_owner(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<String>, DbError> {
        match resource_type {
            "video" => {
                sqlx::query_scalar(
                    "SELECT user_id FROM media_items WHERE id = ? AND media_type = 'video'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "image" => {
                sqlx::query_scalar(
                    "SELECT user_id FROM media_items WHERE id = ? AND media_type = 'image'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "document" => {
                sqlx::query_scalar(
                    "SELECT user_id FROM media_items WHERE id = ? AND media_type = 'document'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "folder" => {
                sqlx::query_scalar("SELECT user_id FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(self.pool())
                    .await
                    .map_err(map_err)
            }
            _ => Ok(None),
        }
    }

    async fn get_resource_group(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<Option<i32>>, DbError> {
        // fetch_optional returns Option<row>; the column itself is Option<i32>.
        // We return Option<Option<i32>>: None = resource not found, Some(None) = no group.
        match resource_type {
            "video" => {
                sqlx::query_scalar(
                    "SELECT group_id FROM media_items WHERE id = ? AND media_type = 'video'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "image" => {
                sqlx::query_scalar(
                    "SELECT group_id FROM media_items WHERE id = ? AND media_type = 'image'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "document" => {
                sqlx::query_scalar(
                    "SELECT group_id FROM media_items WHERE id = ? AND media_type = 'document'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "folder" => {
                sqlx::query_scalar("SELECT group_id FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(self.pool())
                    .await
                    .map_err(map_err)
            }
            _ => Ok(None),
        }
    }

    async fn get_user_group_role(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<String>, DbError> {
        sqlx::query_scalar("SELECT role FROM group_members WHERE user_id = ? AND group_id = ?")
            .bind(user_id)
            .bind(group_id)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)
    }

    async fn get_access_key_data(
        &self,
        key: &str,
    ) -> Result<Option<AccessKeyRow>, DbError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            key: String,
            description: String,
            permission_level: String,
            access_group_id: Option<i32>,
            share_all_group_resources: bool,
            expires_at: Option<String>,
            max_downloads: Option<i32>,
            current_downloads: i32,
            is_active: bool,
        }

        let row: Option<Row> = sqlx::query_as(
            "SELECT
                id,
                code AS key,
                description,
                permission_level,
                access_group_id,
                share_all_group_resources,
                expires_at,
                max_downloads,
                current_downloads,
                is_active
             FROM access_codes
             WHERE code = ? AND is_active = 1",
        )
        .bind(key)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| AccessKeyRow {
            id: r.id,
            key: r.key,
            description: r.description,
            permission_level: r.permission_level,
            access_group_id: r.access_group_id,
            share_all_group_resources: r.share_all_group_resources,
            expires_at: r.expires_at,
            max_downloads: r.max_downloads,
            current_downloads: r.current_downloads,
            is_active: r.is_active,
        }))
    }

    async fn get_resource_slug(
        &self,
        resource_id: i32,
        media_type: &str,
    ) -> Result<Option<String>, DbError> {
        sqlx::query_scalar("SELECT slug FROM media_items WHERE id = ? AND media_type = ?")
            .bind(resource_id)
            .bind(media_type)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)
    }

    async fn access_code_has_permission(
        &self,
        access_code_id: i32,
        media_type: &str,
        media_slug: &str,
    ) -> Result<bool, DbError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM access_code_permissions
                WHERE access_code_id = ?
                  AND media_type = ?
                  AND media_slug = ?
            )",
        )
        .bind(access_code_id)
        .bind(media_type)
        .bind(media_slug)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        Ok(exists)
    }

    async fn increment_download_count(&self, key: &str) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE access_codes
             SET current_downloads = current_downloads + 1,
                 last_accessed_at = CURRENT_TIMESTAMP
             WHERE code = ?",
        )
        .bind(key)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(())
    }

    async fn resource_exists(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<bool, DbError> {
        let exists: bool = match resource_type {
            "video" => {
                sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM media_items WHERE id = ? AND media_type = 'video')",
                )
                .bind(resource_id)
                .fetch_one(self.pool())
                .await
                .map_err(map_err)?
            }
            "image" => {
                sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM media_items WHERE id = ? AND media_type = 'image')",
                )
                .bind(resource_id)
                .fetch_one(self.pool())
                .await
                .map_err(map_err)?
            }
            "document" => {
                sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM media_items WHERE id = ? AND media_type = 'document')",
                )
                .bind(resource_id)
                .fetch_one(self.pool())
                .await
                .map_err(map_err)?
            }
            "folder" => {
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM folders WHERE id = ?)")
                    .bind(resource_id)
                    .fetch_one(self.pool())
                    .await
                    .map_err(map_err)?
            }
            _ => false,
        };
        Ok(exists)
    }

    async fn get_resource_title(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<String>, DbError> {
        match resource_type {
            "video" => {
                sqlx::query_scalar(
                    "SELECT title FROM media_items WHERE id = ? AND media_type = 'video'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "image" => {
                sqlx::query_scalar(
                    "SELECT title FROM media_items WHERE id = ? AND media_type = 'image'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "document" => {
                sqlx::query_scalar(
                    "SELECT title FROM media_items WHERE id = ? AND media_type = 'document'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "folder" => {
                sqlx::query_scalar("SELECT name FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(self.pool())
                    .await
                    .map_err(map_err)
            }
            _ => Ok(None),
        }
    }

    async fn is_user_in_group(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<bool, DbError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM group_members
                WHERE user_id = ? AND group_id = ?
            )",
        )
        .bind(user_id)
        .bind(group_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        Ok(exists)
    }

    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<i32>, DbError> {
        sqlx::query_scalar(
            "SELECT group_id FROM group_members WHERE user_id = ? ORDER BY group_id",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)
    }

    async fn batch_check_public(
        &self,
        resource_type: &str,
        resource_ids: &[i32],
    ) -> Result<Vec<(i32, bool)>, DbError> {
        if resource_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = resource_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");

        let query = match resource_type {
            "video" => format!(
                "SELECT id, is_public FROM media_items WHERE media_type = 'video' AND id IN ({})",
                placeholders
            ),
            "image" => format!(
                "SELECT id, is_public FROM media_items WHERE media_type = 'image' AND id IN ({})",
                placeholders
            ),
            "document" => format!(
                "SELECT id, is_public FROM media_items WHERE media_type = 'document' AND id IN ({})",
                placeholders
            ),
            "folder" => format!(
                "SELECT id, is_public FROM folders WHERE id IN ({})",
                placeholders
            ),
            _ => return Ok(vec![]),
        };

        let mut q = sqlx::query_as(&query);
        for id in resource_ids {
            q = q.bind(id);
        }

        q.fetch_all(self.pool()).await.map_err(map_err)
    }

    async fn get_resource_visibility(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<Option<String>, DbError> {
        match resource_type {
            "video" => {
                sqlx::query_scalar(
                    "SELECT status FROM media_items WHERE id = ? AND media_type = 'video'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "image" => {
                sqlx::query_scalar(
                    "SELECT status FROM media_items WHERE id = ? AND media_type = 'image'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "document" => {
                sqlx::query_scalar(
                    "SELECT status FROM media_items WHERE id = ? AND media_type = 'document'",
                )
                .bind(resource_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)
            }
            "folder" => {
                sqlx::query_scalar("SELECT visibility FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(self.pool())
                    .await
                    .map_err(map_err)
            }
            _ => Ok(None),
        }
    }
}

// ── AuditRepository ────────────────────────────────────────────────

#[async_trait::async_trait]
impl AuditRepository for SqliteDatabase {
    async fn log_entry(&self, entry: &AuditInsert) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO access_audit_log (
                user_id, access_key, ip_address, user_agent,
                resource_type, resource_id,
                permission_requested, permission_granted,
                access_granted, access_layer, reason
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&entry.user_id)
        .bind(&entry.access_key)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(&entry.resource_type)
        .bind(entry.resource_id)
        .bind(&entry.permission_requested)
        .bind(&entry.permission_granted)
        .bind(entry.access_granted)
        .bind(&entry.access_layer)
        .bind(&entry.reason)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(())
    }

    async fn get_resource_audit_log(
        &self,
        resource_type: &str,
        resource_id: i32,
        limit: i32,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            user_id: Option<String>,
            access_key: Option<String>,
            ip_address: Option<String>,
            user_agent: Option<String>,
            resource_type: String,
            resource_id: i32,
            permission_requested: String,
            permission_granted: Option<String>,
            access_granted: bool,
            access_layer: String,
            reason: String,
            created_at: String,
        }

        let rows: Vec<Row> = sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(limit)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(|r| AuditLogRow {
            id: r.id,
            user_id: r.user_id,
            access_key: r.access_key,
            ip_address: r.ip_address,
            user_agent: r.user_agent,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            permission_requested: r.permission_requested,
            permission_granted: r.permission_granted,
            access_granted: r.access_granted,
            access_layer: r.access_layer,
            reason: r.reason,
            created_at: r.created_at,
        }).collect())
    }

    async fn get_denied_attempts(
        &self,
        since_iso: &str,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            user_id: Option<String>,
            access_key: Option<String>,
            ip_address: Option<String>,
            user_agent: Option<String>,
            resource_type: String,
            resource_id: i32,
            permission_requested: String,
            permission_granted: Option<String>,
            access_granted: bool,
            access_layer: String,
            reason: String,
            created_at: String,
        }

        let rows: Vec<Row> = sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE access_granted = 0 AND datetime(created_at) >= datetime(?)
             ORDER BY created_at DESC",
        )
        .bind(since_iso)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(|r| AuditLogRow {
            id: r.id,
            user_id: r.user_id,
            access_key: r.access_key,
            ip_address: r.ip_address,
            user_agent: r.user_agent,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            permission_requested: r.permission_requested,
            permission_granted: r.permission_granted,
            access_granted: r.access_granted,
            access_layer: r.access_layer,
            reason: r.reason,
            created_at: r.created_at,
        }).collect())
    }

    async fn get_denied_by_ip(
        &self,
        ip_address: &str,
        since_iso: &str,
    ) -> Result<Vec<AuditLogRow>, DbError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            user_id: Option<String>,
            access_key: Option<String>,
            ip_address: Option<String>,
            user_agent: Option<String>,
            resource_type: String,
            resource_id: i32,
            permission_requested: String,
            permission_granted: Option<String>,
            access_granted: bool,
            access_layer: String,
            reason: String,
            created_at: String,
        }

        let rows: Vec<Row> = sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE access_granted = 0
               AND ip_address = ?
               AND datetime(created_at) >= datetime(?)
             ORDER BY created_at DESC",
        )
        .bind(ip_address)
        .bind(since_iso)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows.into_iter().map(|r| AuditLogRow {
            id: r.id,
            user_id: r.user_id,
            access_key: r.access_key,
            ip_address: r.ip_address,
            user_agent: r.user_agent,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            permission_requested: r.permission_requested,
            permission_granted: r.permission_granted,
            access_granted: r.access_granted,
            access_layer: r.access_layer,
            reason: r.reason,
            created_at: r.created_at,
        }).collect())
    }

    async fn get_user_stats(&self, user_id: &str) -> Result<AuditStats, DbError> {
        let granted: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE user_id = ? AND access_granted = 1",
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        let denied: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE user_id = ? AND access_granted = 0",
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        Ok(AuditStats {
            total_attempts: granted + denied,
            granted_count: granted,
            denied_count: denied,
        })
    }

    async fn get_resource_stats(
        &self,
        resource_type: &str,
        resource_id: i32,
    ) -> Result<AuditStats, DbError> {
        let granted: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ? AND access_granted = 1",
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        let denied: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ? AND access_granted = 0",
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        Ok(AuditStats {
            total_attempts: granted + denied,
            granted_count: granted,
            denied_count: denied,
        })
    }

    async fn check_failed_attempts(
        &self,
        ip_address: &str,
        window_minutes: i32,
    ) -> Result<i32, DbError> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE ip_address = ?
               AND access_granted = 0
               AND created_at > datetime('now', '-' || ? || ' minutes')",
        )
        .bind(ip_address)
        .bind(window_minutes)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)
    }

    async fn cleanup_old_logs(&self, older_than_iso: &str) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM access_audit_log WHERE created_at < ?")
            .bind(older_than_iso)
            .execute(self.pool())
            .await
            .map_err(map_err)?;

        Ok(result.rows_affected())
    }
}

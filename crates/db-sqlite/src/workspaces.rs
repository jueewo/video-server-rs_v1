use crate::SqliteDatabase;
use db::workspaces::*;
use db::DbError;
use std::collections::HashMap;

fn map_err(e: sqlx::Error) -> DbError {
    match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE constraint") => {
            DbError::UniqueViolation(db_err.message().to_string())
        }
        _ => DbError::Internal(e.to_string()),
    }
}

#[async_trait::async_trait]
impl WorkspaceRepository for SqliteDatabase {
    // ── Workspace CRUD ───────────────────────────────────────────

    async fn insert_workspace(
        &self,
        workspace_id: &str,
        user_id: &str,
        tenant_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO workspaces (workspace_id, user_id, tenant_id, name, description, created_at) \
             VALUES (?, ?, ?, ?, ?, datetime('now'))",
        )
        .bind(workspace_id)
        .bind(user_id)
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn update_workspace(
        &self,
        workspace_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE workspaces SET name = COALESCE(?, name), description = COALESCE(?, description), \
             updated_at = datetime('now') WHERE workspace_id = ?",
        )
        .bind(name)
        .bind(description)
        .bind(workspace_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete_workspace(&self, workspace_id: &str, user_id: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM workspaces WHERE workspace_id = ? AND user_id = ?")
            .bind(workspace_id)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn verify_workspace_ownership(
        &self,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<Option<(String, Option<String>)>, DbError> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT name, description FROM workspaces WHERE workspace_id = ? AND user_id = ?",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row)
    }

    async fn list_user_workspaces(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<Vec<WorkspaceRow>, DbError> {
        let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT workspace_id, name, description, created_at FROM workspaces \
             WHERE user_id = ? AND (tenant_id = ? OR tenant_id IS NULL) \
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(workspace_id, name, description, created_at)| WorkspaceRow {
                workspace_id,
                name,
                description,
                created_at,
            })
            .collect())
    }

    async fn get_workspace_created_at(
        &self,
        workspace_id: &str,
    ) -> Result<Option<String>, DbError> {
        let row: Option<String> =
            sqlx::query_scalar("SELECT created_at FROM workspaces WHERE workspace_id = ?")
                .bind(workspace_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(row)
    }

    async fn get_workspace_owner(
        &self,
        workspace_id: &str,
    ) -> Result<Option<String>, DbError> {
        sqlx::query_scalar("SELECT user_id FROM workspaces WHERE workspace_id = ?")
            .bind(workspace_id)
            .fetch_optional(self.pool())
            .await
            .map_err(map_err)
    }

    // ── Workspace tags ───────────────────────────────────────────

    async fn get_workspace_tags(&self, workspace_id: &str) -> Result<Vec<String>, DbError> {
        sqlx::query_scalar("SELECT tag FROM workspace_tags WHERE workspace_id = ? ORDER BY tag")
            .bind(workspace_id)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)
    }

    async fn set_workspace_tags(
        &self,
        workspace_id: &str,
        tags: &[String],
    ) -> Result<(), DbError> {
        sqlx::query("DELETE FROM workspace_tags WHERE workspace_id = ?")
            .bind(workspace_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;

        for tag in tags.iter().map(|t| t.trim()).filter(|t| !t.is_empty()) {
            sqlx::query(
                "INSERT OR IGNORE INTO workspace_tags (workspace_id, tag) VALUES (?, ?)",
            )
            .bind(workspace_id)
            .bind(tag)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        }
        Ok(())
    }

    async fn get_workspace_tags_for_user(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<HashMap<String, Vec<String>>, DbError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT wt.workspace_id, wt.tag FROM workspace_tags wt \
             JOIN workspaces w ON w.workspace_id = wt.workspace_id \
             WHERE w.user_id = ? AND (w.tenant_id = ? OR w.tenant_id IS NULL)",
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for (ws_id, tag) in rows {
            map.entry(ws_id).or_default().push(tag);
        }
        Ok(map)
    }

    // ── Tenant admin ─────────────────────────────────────────────

    async fn list_tenants(&self) -> Result<Vec<TenantRow>, DbError> {
        let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, name, branding, created_at FROM tenants ORDER BY created_at ASC",
        )
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(id, name, branding_json, created_at)| TenantRow {
                id,
                name,
                branding_json,
                created_at,
            })
            .collect())
    }

    async fn create_tenant(
        &self,
        id: &str,
        name: &str,
        branding_json: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query("INSERT INTO tenants (id, name, branding) VALUES (?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(branding_json)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn list_tenant_users(&self, tenant_id: &str) -> Result<Vec<TenantUserRow>, DbError> {
        let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, email, name, tenant_id FROM users WHERE tenant_id = ? ORDER BY email ASC",
        )
        .bind(tenant_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(user_id, email, name, tenant_id)| TenantUserRow {
                user_id,
                email,
                name,
                tenant_id,
            })
            .collect())
    }

    async fn assign_user_tenant(&self, user_id: &str, tenant_id: &str) -> Result<bool, DbError> {
        let result = sqlx::query("UPDATE users SET tenant_id = ? WHERE id = ?")
            .bind(tenant_id)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn update_tenant_branding(
        &self,
        tenant_id: &str,
        branding_json: &str,
    ) -> Result<bool, DbError> {
        let result = sqlx::query("UPDATE tenants SET branding = ? WHERE id = ?")
            .bind(branding_json)
            .bind(tenant_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_tenant_branding_json(&self, tenant_id: &str) -> Result<Option<String>, DbError> {
        let branding: Option<Option<String>> =
            sqlx::query_scalar("SELECT branding FROM tenants WHERE id = ?")
                .bind(tenant_id)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(branding.flatten())
    }

    async fn create_tenant_invitation(
        &self,
        email: &str,
        tenant_id: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT OR IGNORE INTO tenant_invitations (email, tenant_id) VALUES (?, ?)",
        )
        .bind(email)
        .bind(tenant_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn list_tenant_invitations(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<InvitationRow>, DbError> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT email, tenant_id, invited_at FROM tenant_invitations \
             WHERE tenant_id = ? ORDER BY invited_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(email, tenant_id, invited_at)| InvitationRow {
                email,
                tenant_id,
                invited_at,
            })
            .collect())
    }

    async fn delete_tenant_invitation(
        &self,
        email: &str,
        tenant_id: &str,
    ) -> Result<(), DbError> {
        sqlx::query("DELETE FROM tenant_invitations WHERE email = ? AND tenant_id = ?")
            .bind(email)
            .bind(tenant_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    // ── Workspace access codes ───────────────────────────────────

    async fn workspace_code_grants_access(
        &self,
        code: &str,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<bool, DbError> {
        let row: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM workspace_access_codes wac \
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id \
             WHERE wac.code = ? AND wac.is_active = 1 \
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now')) \
               AND f.workspace_id = ? \
               AND (f.folder_path = ? OR ? LIKE f.folder_path || '/%') \
             LIMIT 1",
        )
        .bind(code)
        .bind(workspace_id)
        .bind(folder_path)
        .bind(folder_path)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.is_some())
    }

    async fn workspace_code_grants_vault_access(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError> {
        // Direct vault grant
        let direct: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM workspace_access_codes wac \
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id \
             WHERE wac.code = ? AND wac.is_active = 1 \
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now')) \
               AND f.vault_id = ? \
             LIMIT 1",
        )
        .bind(code)
        .bind(vault_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        if direct.is_some() {
            return Ok(true);
        }

        // Indirect: code's workspace owner has this vault
        let indirect: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM workspace_access_codes wac \
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id \
             JOIN workspaces w ON f.workspace_id = w.workspace_id \
             JOIN storage_vaults v ON v.user_id = w.user_id \
             WHERE wac.code = ? AND v.vault_id = ? AND f.vault_id IS NULL \
               AND wac.is_active = 1 \
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now')) \
             LIMIT 1",
        )
        .bind(code)
        .bind(vault_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(indirect.is_some())
    }

    async fn create_workspace_access_code(
        &self,
        code: &str,
        created_by: &str,
        description: Option<&str>,
        expires_at: Option<&str>,
        folders: &[FolderGrant],
    ) -> Result<i64, DbError> {
        let code_id: i64 = sqlx::query_scalar(
            "INSERT INTO workspace_access_codes (code, created_by, description, expires_at, is_active, created_at) \
             VALUES (?, ?, ?, ?, 1, datetime('now')) RETURNING id",
        )
        .bind(code)
        .bind(created_by)
        .bind(description)
        .bind(expires_at)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        for grant in folders {
            sqlx::query(
                "INSERT OR IGNORE INTO workspace_access_code_folders \
                 (workspace_access_code_id, workspace_id, folder_path, vault_id, group_id) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(code_id)
            .bind(&grant.workspace_id)
            .bind(&grant.folder_path)
            .bind(&grant.vault_id)
            .bind(&grant.group_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        }

        Ok(code_id)
    }

    async fn list_created_access_codes(
        &self,
        created_by: &str,
    ) -> Result<Vec<CreatedAccessCodeRow>, DbError> {
        let rows: Vec<(String, Option<String>, Option<String>, Option<String>, i64, i64, Option<String>)> =
            sqlx::query_as(
                "SELECT wac.code, wac.description, wac.expires_at, wac.created_at, \
                        wac.is_active, COUNT(f.id) AS folder_count, \
                        GROUP_CONCAT(f.workspace_id || '/' || f.folder_path, '|') AS folder_paths \
                 FROM workspace_access_codes wac \
                 LEFT JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id \
                 WHERE wac.created_by = ? \
                 GROUP BY wac.id \
                 ORDER BY wac.created_at DESC",
            )
            .bind(created_by)
            .fetch_all(self.pool())
            .await
            .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(
                |(code, description, expires_at, created_at, is_active, folder_count, folder_paths)| {
                    CreatedAccessCodeRow {
                        code,
                        description,
                        expires_at,
                        created_at,
                        is_active: is_active != 0,
                        folder_count,
                        folder_paths,
                    }
                },
            )
            .collect())
    }

    async fn list_claimed_access_codes(
        &self,
        user_id: &str,
    ) -> Result<Vec<ClaimedAccessCodeRow>, DbError> {
        let rows: Vec<(String, Option<String>, String, Option<String>)> = sqlx::query_as(
            "SELECT wac.code, wac.description, wac.created_by, ucwc.claimed_at \
             FROM user_claimed_workspace_codes ucwc \
             JOIN workspace_access_codes wac ON wac.id = ucwc.workspace_access_code_id \
             WHERE ucwc.user_id = ? \
             ORDER BY ucwc.claimed_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(code, description, created_by, claimed_at)| ClaimedAccessCodeRow {
                code,
                description,
                created_by,
                claimed_at,
            })
            .collect())
    }

    async fn update_workspace_access_code(
        &self,
        code: &str,
        created_by: &str,
        description: Option<&str>,
        expires_at: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<bool, DbError> {
        let result = if let Some(active) = is_active {
            sqlx::query(
                "UPDATE workspace_access_codes \
                 SET description = COALESCE(?, description), \
                     expires_at = COALESCE(?, expires_at), \
                     is_active = ? \
                 WHERE code = ? AND created_by = ?",
            )
            .bind(description)
            .bind(expires_at)
            .bind(if active { 1i64 } else { 0 })
            .bind(code)
            .bind(created_by)
            .execute(self.pool())
            .await
            .map_err(map_err)?
        } else {
            sqlx::query(
                "UPDATE workspace_access_codes \
                 SET description = COALESCE(?, description), \
                     expires_at = COALESCE(?, expires_at) \
                 WHERE code = ? AND created_by = ?",
            )
            .bind(description)
            .bind(expires_at)
            .bind(code)
            .bind(created_by)
            .execute(self.pool())
            .await
            .map_err(map_err)?
        };
        Ok(result.rows_affected() > 0)
    }

    async fn delete_workspace_access_code(
        &self,
        code: &str,
        created_by: &str,
    ) -> Result<bool, DbError> {
        let code_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes WHERE code = ? AND created_by = ?",
        )
        .bind(code)
        .bind(created_by)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let code_id = match code_id {
            Some(id) => id,
            None => return Ok(false),
        };

        sqlx::query("DELETE FROM workspace_access_code_folders WHERE workspace_access_code_id = ?")
            .bind(code_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;

        sqlx::query(
            "DELETE FROM user_claimed_workspace_codes WHERE workspace_access_code_id = ?",
        )
        .bind(code_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        sqlx::query("DELETE FROM workspace_access_codes WHERE id = ?")
            .bind(code_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;

        Ok(true)
    }

    async fn claim_workspace_access_code(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<bool, DbError> {
        let code_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes \
             WHERE code = ? AND is_active = 1 \
               AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let code_id = match code_id {
            Some(id) => id,
            None => return Ok(false),
        };

        sqlx::query(
            "INSERT OR IGNORE INTO user_claimed_workspace_codes \
             (user_id, workspace_access_code_id, claimed_at) \
             VALUES (?, ?, datetime('now'))",
        )
        .bind(user_id)
        .bind(code_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(true)
    }

    async fn unclaim_workspace_access_code(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<bool, DbError> {
        let code_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes WHERE code = ?",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let code_id = match code_id {
            Some(id) => id,
            None => return Ok(false),
        };

        let result = sqlx::query(
            "DELETE FROM user_claimed_workspace_codes \
             WHERE user_id = ? AND workspace_access_code_id = ?",
        )
        .bind(user_id)
        .bind(code_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn add_folder_to_access_code(
        &self,
        code: &str,
        created_by: &str,
        grant: &FolderGrant,
    ) -> Result<bool, DbError> {
        let code_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes WHERE code = ? AND created_by = ?",
        )
        .bind(code)
        .bind(created_by)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let code_id = match code_id {
            Some(id) => id,
            None => return Ok(false),
        };

        // Verify caller owns the workspace
        let owns: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM workspaces WHERE workspace_id = ? AND user_id = ?",
        )
        .bind(&grant.workspace_id)
        .bind(created_by)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        if owns.is_none() {
            return Ok(false);
        }

        sqlx::query(
            "INSERT OR IGNORE INTO workspace_access_code_folders \
             (workspace_access_code_id, workspace_id, folder_path, vault_id, group_id) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(code_id)
        .bind(&grant.workspace_id)
        .bind(&grant.folder_path)
        .bind(&grant.vault_id)
        .bind(&grant.group_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(true)
    }

    async fn remove_folder_from_access_code(
        &self,
        code: &str,
        created_by: &str,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<bool, DbError> {
        let code_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes WHERE code = ? AND created_by = ?",
        )
        .bind(code)
        .bind(created_by)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let code_id = match code_id {
            Some(id) => id,
            None => return Ok(false),
        };

        let result = sqlx::query(
            "DELETE FROM workspace_access_code_folders \
             WHERE workspace_access_code_id = ? AND workspace_id = ? AND folder_path = ?",
        )
        .bind(code_id)
        .bind(workspace_id)
        .bind(folder_path)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_access_code_folder_files(
        &self,
        code: &str,
    ) -> Result<Vec<(String, String)>, DbError> {
        // Only non-media-server folder grants (vault_id IS NULL)
        let code_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM workspace_access_codes \
             WHERE code = ? AND is_active = 1 \
               AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )
        .bind(code)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        let code_id = match code_id {
            Some(id) => id,
            None => return Ok(vec![]),
        };

        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT workspace_id, folder_path FROM workspace_access_code_folders \
             WHERE workspace_access_code_id = ? AND vault_id IS NULL",
        )
        .bind(code_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows)
    }

    // ── Access code folder path maintenance ──────────────────────

    async fn delete_access_code_folders_for_path(
        &self,
        workspace_id: &str,
        path: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "DELETE FROM workspace_access_code_folders \
             WHERE workspace_id = ? \
               AND (folder_path = ? OR folder_path LIKE ? || '/%')",
        )
        .bind(workspace_id)
        .bind(path)
        .bind(path)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn rename_access_code_folder_paths(
        &self,
        workspace_id: &str,
        old_path: &str,
        new_path: &str,
    ) -> Result<(), DbError> {
        // Exact match
        sqlx::query(
            "UPDATE workspace_access_code_folders \
             SET folder_path = ? \
             WHERE workspace_id = ? AND folder_path = ?",
        )
        .bind(new_path)
        .bind(workspace_id)
        .bind(old_path)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        // Sub-paths: replace old prefix with new
        sqlx::query(
            "UPDATE workspace_access_code_folders \
             SET folder_path = ? || substr(folder_path, ? + 1) \
             WHERE workspace_id = ? AND folder_path LIKE ? || '/%'",
        )
        .bind(new_path)
        .bind(old_path.len() as i64)
        .bind(workspace_id)
        .bind(old_path)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(())
    }

    // ── Cross-domain helpers ─────────────────────────────────────

    async fn media_slug_exists(&self, slug: &str) -> Result<Option<i64>, DbError> {
        let id: Option<i64> =
            sqlx::query_scalar("SELECT id FROM media_items WHERE slug = ?")
                .bind(slug)
                .fetch_optional(self.pool())
                .await
                .map_err(map_err)?;
        Ok(id)
    }

    async fn insert_published_media(
        &self,
        slug: &str,
        media_type: &str,
        title: &str,
        filename: &str,
        original_filename: &str,
        mime_type: &str,
        file_size: i64,
        user_id: &str,
        vault_id: &str,
        tenant_id: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO media_items \
             (slug, media_type, title, filename, original_filename, mime_type, file_size, \
              is_public, user_id, vault_id, status, allow_download, allow_comments, mature_content, tenant_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, 'active', 1, 1, 0, ?)",
        )
        .bind(slug)
        .bind(media_type)
        .bind(title)
        .bind(filename)
        .bind(original_filename)
        .bind(mime_type)
        .bind(file_size)
        .bind(user_id)
        .bind(vault_id)
        .bind(tenant_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn verify_vault_ownership(
        &self,
        vault_id: &str,
        user_id: &str,
    ) -> Result<Option<String>, DbError> {
        let id: Option<String> = sqlx::query_scalar(
            "SELECT vault_id FROM storage_vaults WHERE vault_id = ? AND user_id = ?",
        )
        .bind(vault_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(id)
    }

    async fn media_exists_in_vault(
        &self,
        slug: &str,
        vault_id: &str,
    ) -> Result<bool, DbError> {
        let row: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM media_items WHERE slug = ? AND vault_id = ?",
        )
        .bind(slug)
        .bind(vault_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.is_some())
    }

    async fn insert_access_code(
        &self,
        code: &str,
        created_by: &str,
    ) -> Result<i64, DbError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO access_codes (code, created_by, permission_level, is_active, created_at) \
             VALUES (?, ?, 'read', 1, datetime('now')) RETURNING id",
        )
        .bind(code)
        .bind(created_by)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;
        Ok(id)
    }

    async fn insert_access_code_permission(
        &self,
        access_code_id: i64,
        media_type: &str,
        media_slug: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO access_code_permissions (access_code_id, media_type, media_slug) \
             VALUES (?, ?, ?)",
        )
        .bind(access_code_id)
        .bind(media_type)
        .bind(media_slug)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn get_folder_grants_for_code(
        &self,
        code: &str,
    ) -> Result<Vec<(String, String)>, DbError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT f.workspace_id, f.folder_path
             FROM workspace_access_codes wac
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             WHERE wac.code = ? AND wac.is_active = 1
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
               AND f.vault_id IS NULL
             ORDER BY f.folder_path",
        )
        .bind(code)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows)
    }

    async fn get_preview_code_for_folder(
        &self,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<Option<String>, DbError> {
        let code: Option<String> = sqlx::query_scalar(
            "SELECT wac.code
             FROM workspace_access_codes wac
             JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
             WHERE f.workspace_id = ? AND f.folder_path = ? AND f.vault_id IS NULL
               AND wac.is_active = 1
               AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
             LIMIT 1",
        )
        .bind(workspace_id)
        .bind(folder_path)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(code)
    }
}

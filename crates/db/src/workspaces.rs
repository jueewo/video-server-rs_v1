//! Workspace repository trait and domain types.

use crate::DbError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Workspace CRUD types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRow {
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

// ── Tenant types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TenantRow {
    pub id: String,
    pub name: String,
    pub branding_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantUserRow {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvitationRow {
    pub email: String,
    pub tenant_id: String,
    pub invited_at: String,
}

// ── Workspace access code types ──────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CreatedAccessCodeRow {
    pub code: String,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: Option<String>,
    pub is_active: bool,
    pub folder_count: i64,
    /// Pipe-separated "workspace_id/folder_path" entries.
    pub folder_paths: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaimedAccessCodeRow {
    pub code: String,
    pub description: Option<String>,
    pub created_by: String,
    pub claimed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FolderGrant {
    pub workspace_id: String,
    pub folder_path: String,
    pub vault_id: Option<String>,
    pub group_id: Option<String>,
}

// ── Repository trait ─────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait WorkspaceRepository: Send + Sync {
    // ── Workspace CRUD ───────────────────────────────────────────

    async fn insert_workspace(
        &self,
        workspace_id: &str,
        user_id: &str,
        tenant_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), DbError>;

    async fn update_workspace(
        &self,
        workspace_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), DbError>;

    async fn delete_workspace(&self, workspace_id: &str, user_id: &str) -> Result<(), DbError>;

    /// Returns (name, description) if workspace belongs to user, else None.
    async fn verify_workspace_ownership(
        &self,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<Option<(String, Option<String>)>, DbError>;

    async fn list_user_workspaces(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<Vec<WorkspaceRow>, DbError>;

    async fn get_workspace_created_at(
        &self,
        workspace_id: &str,
    ) -> Result<Option<String>, DbError>;

    /// Get the owner user_id of a workspace.
    async fn get_workspace_owner(
        &self,
        workspace_id: &str,
    ) -> Result<Option<String>, DbError>;

    // ── Workspace tags ───────────────────────────────────────────

    /// Get tags for a single workspace.
    async fn get_workspace_tags(&self, workspace_id: &str) -> Result<Vec<String>, DbError>;

    async fn set_workspace_tags(
        &self,
        workspace_id: &str,
        tags: &[String],
    ) -> Result<(), DbError>;

    /// Returns workspace_id → Vec<tag> for all of a user's workspaces.
    async fn get_workspace_tags_for_user(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<HashMap<String, Vec<String>>, DbError>;

    // ── Tenant admin ─────────────────────────────────────────────

    async fn list_tenants(&self) -> Result<Vec<TenantRow>, DbError>;

    async fn create_tenant(
        &self,
        id: &str,
        name: &str,
        branding_json: Option<&str>,
    ) -> Result<(), DbError>;

    async fn list_tenant_users(&self, tenant_id: &str) -> Result<Vec<TenantUserRow>, DbError>;

    /// Returns false if user not found.
    async fn assign_user_tenant(&self, user_id: &str, tenant_id: &str) -> Result<bool, DbError>;

    /// Returns false if tenant not found.
    async fn update_tenant_branding(
        &self,
        tenant_id: &str,
        branding_json: &str,
    ) -> Result<bool, DbError>;

    async fn get_tenant_branding_json(&self, tenant_id: &str) -> Result<Option<String>, DbError>;

    async fn create_tenant_invitation(
        &self,
        email: &str,
        tenant_id: &str,
    ) -> Result<(), DbError>;

    async fn list_tenant_invitations(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<InvitationRow>, DbError>;

    async fn delete_tenant_invitation(
        &self,
        email: &str,
        tenant_id: &str,
    ) -> Result<(), DbError>;

    // ── Workspace access codes ───────────────────────────────────

    /// Checks if a workspace access code grants access to a specific folder.
    async fn workspace_code_grants_access(
        &self,
        code: &str,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<bool, DbError>;

    /// Checks if a workspace access code grants access to a vault.
    async fn workspace_code_grants_vault_access(
        &self,
        code: &str,
        vault_id: &str,
    ) -> Result<bool, DbError>;

    /// Creates a workspace access code with folder grants. Returns the code id.
    async fn create_workspace_access_code(
        &self,
        code: &str,
        created_by: &str,
        description: Option<&str>,
        expires_at: Option<&str>,
        folders: &[FolderGrant],
    ) -> Result<i64, DbError>;

    async fn list_created_access_codes(
        &self,
        created_by: &str,
    ) -> Result<Vec<CreatedAccessCodeRow>, DbError>;

    async fn list_claimed_access_codes(
        &self,
        user_id: &str,
    ) -> Result<Vec<ClaimedAccessCodeRow>, DbError>;

    async fn update_workspace_access_code(
        &self,
        code: &str,
        created_by: &str,
        description: Option<&str>,
        expires_at: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<bool, DbError>;

    async fn delete_workspace_access_code(
        &self,
        code: &str,
        created_by: &str,
    ) -> Result<bool, DbError>;

    async fn claim_workspace_access_code(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<bool, DbError>;

    async fn unclaim_workspace_access_code(
        &self,
        code: &str,
        user_id: &str,
    ) -> Result<bool, DbError>;

    async fn add_folder_to_access_code(
        &self,
        code: &str,
        created_by: &str,
        grant: &FolderGrant,
    ) -> Result<bool, DbError>;

    async fn remove_folder_from_access_code(
        &self,
        code: &str,
        created_by: &str,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<bool, DbError>;

    /// Returns (workspace_id, folder_path) pairs for non-media-server folder grants.
    async fn get_access_code_folder_files(
        &self,
        code: &str,
    ) -> Result<Vec<(String, String)>, DbError>;

    // ── Access code folder path maintenance ──────────────────────

    async fn delete_access_code_folders_for_path(
        &self,
        workspace_id: &str,
        path: &str,
    ) -> Result<(), DbError>;

    async fn rename_access_code_folder_paths(
        &self,
        workspace_id: &str,
        old_path: &str,
        new_path: &str,
    ) -> Result<(), DbError>;

    // ── Cross-domain helpers (used by publishing) ────────────────

    /// Check if a media slug exists. Returns Some(id) if found.
    async fn media_slug_exists(&self, slug: &str) -> Result<Option<i64>, DbError>;

    /// Insert a media_items record for published workspace files.
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
    ) -> Result<(), DbError>;

    /// Check if a vault belongs to a user. Returns Some(vault_id) if found.
    async fn verify_vault_ownership(
        &self,
        vault_id: &str,
        user_id: &str,
    ) -> Result<Option<String>, DbError>;

    /// Check media reference exists in vault (for course publishing validation).
    async fn media_exists_in_vault(
        &self,
        slug: &str,
        vault_id: &str,
    ) -> Result<bool, DbError>;

    /// Insert access code for course publishing. Returns the code id.
    async fn insert_access_code(
        &self,
        code: &str,
        created_by: &str,
    ) -> Result<i64, DbError>;

    /// Insert access code permission for course publishing.
    async fn insert_access_code_permission(
        &self,
        access_code_id: i64,
        media_type: &str,
        media_slug: &str,
    ) -> Result<(), DbError>;

    /// Get non-vault folder grants for an access code.
    /// Returns (workspace_id, folder_path) pairs for active, non-expired codes
    /// where vault_id IS NULL.
    async fn get_folder_grants_for_code(
        &self,
        code: &str,
    ) -> Result<Vec<(String, String)>, DbError>;

    /// Get an active preview access code for a specific workspace folder.
    /// Returns the first active, non-expired code where vault_id IS NULL.
    async fn get_preview_code_for_folder(
        &self,
        workspace_id: &str,
        folder_path: &str,
    ) -> Result<Option<String>, DbError>;
}

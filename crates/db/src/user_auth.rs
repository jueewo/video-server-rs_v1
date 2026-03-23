//! User authentication repository trait and domain types.

use crate::DbError;

/// Login info fetched from the users table.
#[derive(Debug, Clone)]
pub struct UserLoginInfo {
    pub provider: String,
    pub last_login_at: Option<String>,
}

/// Parameters for upserting a user (OIDC or emergency).
#[derive(Debug)]
pub struct UpsertUserRequest<'a> {
    pub id: &'a str,
    pub email: &'a str,
    pub name: &'a str,
    pub avatar_url: Option<&'a str>,
    pub provider: &'a str,
}

#[async_trait::async_trait]
pub trait UserAuthRepository: Send + Sync {
    /// Upsert a user record. On conflict updates email/name/avatar and last_login_at.
    async fn upsert_user(&self, req: &UpsertUserRequest<'_>) -> Result<(), DbError>;

    /// Get provider and last_login_at for a user.
    async fn get_user_login_info(&self, user_id: &str) -> Result<Option<UserLoginInfo>, DbError>;

    /// Get the tenant_id for a user. Returns None if user not found.
    async fn get_user_tenant_id(&self, user_id: &str) -> Result<Option<String>, DbError>;

    /// Set the tenant_id on a user row.
    async fn set_user_tenant(&self, user_id: &str, tenant_id: &str) -> Result<(), DbError>;

    /// Look up a pending tenant invitation by email. Returns the tenant_id if found.
    async fn get_tenant_invitation_by_email(&self, email: &str) -> Result<Option<String>, DbError>;

    /// Delete a tenant invitation by email (consume it).
    async fn delete_tenant_invitation(&self, email: &str) -> Result<(), DbError>;

    /// Get the raw branding JSON string for a tenant. Returns None if tenant not found or no branding.
    async fn get_tenant_branding_json(&self, tenant_id: &str) -> Result<Option<String>, DbError>;

    /// Get a user's email by id. Returns None if user not found.
    async fn get_user_email(&self, user_id: &str) -> Result<Option<String>, DbError>;

    /// Get a user's display name by id. Returns None if user not found.
    async fn get_user_name(&self, user_id: &str) -> Result<Option<Option<String>>, DbError>;
}

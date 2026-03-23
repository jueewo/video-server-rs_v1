//! Access group repository trait and domain types.

use crate::DbError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Access group model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessGroup {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: i32,
    pub settings: Option<String>,
}

impl AccessGroup {
    pub fn is_active(&self) -> bool {
        self.is_active != 0
    }

    pub fn created_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }

    pub fn updated_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }
}

/// Group member model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: i32,
    pub group_id: i32,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
    pub invited_by: Option<String>,
}

impl GroupMember {
    pub fn joined_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.joined_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.joined_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }
}

/// Group invitation model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInvitation {
    pub id: i32,
    pub group_id: i32,
    pub email: String,
    pub token: String,
    pub role: String,
    pub invited_by: String,
    pub created_at: String,
    pub expires_at: String,
    pub accepted_at: Option<String>,
    pub accepted_by: Option<String>,
}

impl GroupInvitation {
    pub fn created_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }

    pub fn expires_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.expires_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.expires_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }

    pub fn accepted_at_datetime(&self) -> Option<Result<DateTime<Utc>, chrono::ParseError>> {
        self.accepted_at.as_ref().map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                })
        })
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at_datetime() {
            Ok(expires_at) => Utc::now() > expires_at,
            Err(_) => false,
        }
    }

    pub fn is_accepted(&self) -> bool {
        self.accepted_at.is_some()
    }
}

/// Group with additional metadata (member/media counts, user's role).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithMetadata {
    #[serde(flatten)]
    pub group: AccessGroup,
    pub member_count: i32,
    pub media_count: i64,
    /// The requesting user's role as a string (e.g. "owner", "admin", "viewer").
    pub user_role: Option<String>,
    pub is_owner: bool,
}

/// Member with user information (name + email from users table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWithUser {
    #[serde(flatten)]
    pub member: GroupMember,
    pub name: String,
    pub email: Option<String>,
}

#[async_trait::async_trait]
pub trait AccessGroupRepository: Send + Sync {
    // ── Groups ──────────────────────────────────────────────────────

    /// Check if a slug already exists.
    async fn slug_exists(&self, slug: &str) -> Result<bool, DbError>;

    /// Insert a new group and add the owner as a member. Returns the group.
    async fn insert_group(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        owner_id: &str,
    ) -> Result<AccessGroup, DbError>;

    /// Get an active group by ID.
    async fn get_group_by_id(&self, id: i32) -> Result<Option<AccessGroup>, DbError>;

    /// Get an active group by slug.
    async fn get_group_by_slug(&self, slug: &str) -> Result<Option<AccessGroup>, DbError>;

    /// Get all groups a user belongs to, with metadata.
    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<GroupWithMetadata>, DbError>;

    /// Update group name and description.
    async fn update_group(&self, id: i32, name: &str, description: Option<&str>) -> Result<(), DbError>;

    /// Soft-delete a group (set is_active = 0).
    async fn soft_delete_group(&self, id: i32) -> Result<(), DbError>;

    // ── Members ─────────────────────────────────────────────────────

    /// Check if a user is a member of a group.
    async fn is_group_member(&self, group_id: i32, user_id: &str) -> Result<bool, DbError>;

    /// Get a user's role in a group. Returns None if not a member.
    async fn get_user_role(&self, group_id: i32, user_id: &str) -> Result<Option<String>, DbError>;

    /// Get all members of a group with user info (name, email).
    async fn get_group_members(&self, group_id: i32) -> Result<Vec<MemberWithUser>, DbError>;

    /// Add a member to a group. Returns the new member record.
    async fn add_member(
        &self,
        group_id: i32,
        user_id: &str,
        role: &str,
        invited_by: Option<&str>,
    ) -> Result<GroupMember, DbError>;

    /// Remove a member from a group. Returns true if deleted.
    async fn remove_member(&self, group_id: i32, user_id: &str) -> Result<bool, DbError>;

    /// Update a member's role. Returns the updated member, or None if not found.
    async fn update_member_role(
        &self,
        group_id: i32,
        user_id: &str,
        role: &str,
    ) -> Result<Option<GroupMember>, DbError>;

    /// Count how many owners a group has.
    async fn count_owners(&self, group_id: i32) -> Result<i32, DbError>;

    // ── Invitations ─────────────────────────────────────────────────

    /// Create an invitation. Returns the invitation record.
    async fn create_invitation(
        &self,
        group_id: i32,
        email: &str,
        token: &str,
        role: &str,
        invited_by: &str,
        expires_at: &str,
    ) -> Result<GroupInvitation, DbError>;

    /// Get an invitation by its token.
    async fn get_invitation_by_token(&self, token: &str) -> Result<Option<GroupInvitation>, DbError>;

    /// Get all pending (unaccepted) invitations for a group.
    async fn get_group_invitations(&self, group_id: i32) -> Result<Vec<GroupInvitation>, DbError>;

    /// Mark an invitation as accepted.
    async fn mark_invitation_accepted(&self, invitation_id: i32, user_id: &str) -> Result<(), DbError>;

    /// Delete an invitation.
    async fn delete_invitation(&self, invitation_id: i32) -> Result<(), DbError>;
}

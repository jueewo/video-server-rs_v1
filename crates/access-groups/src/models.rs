//! Database models for access groups

use chrono::{DateTime, Utc};
use common::types::GroupRole;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Access group model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessGroup {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: i32,           // SQLite stores BOOLEAN as INTEGER (0 or 1)
    pub settings: Option<String>, // JSON
}

impl AccessGroup {
    /// Check if group is active (helper method)
    pub fn is_active(&self) -> bool {
        self.is_active != 0
    }
}

impl AccessGroup {
    /// Parse created_at as DateTime
    pub fn created_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Try SQLite format
                chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }

    /// Parse updated_at as DateTime
    pub fn updated_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Try SQLite format
                chrono::NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }
}

/// Group member model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub id: i32,
    pub group_id: i32,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
    pub invited_by: Option<String>,
}

impl GroupMember {
    /// Get role as GroupRole enum
    pub fn role_enum(&self) -> Result<GroupRole, String> {
        self.role.parse()
    }

    /// Parse joined_at as DateTime
    pub fn joined_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.joined_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Try SQLite format
                chrono::NaiveDateTime::parse_from_str(&self.joined_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }
}

/// Group invitation model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    /// Get role as GroupRole enum
    pub fn role_enum(&self) -> Result<GroupRole, String> {
        self.role.parse()
    }

    /// Parse created_at as DateTime
    pub fn created_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }

    /// Parse expires_at as DateTime
    pub fn expires_at_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.expires_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.expires_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            })
    }

    /// Parse accepted_at as DateTime if present
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

    /// Check if invitation is expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at_datetime() {
            Ok(expires_at) => Utc::now() > expires_at,
            Err(_) => false,
        }
    }

    /// Check if invitation is accepted
    pub fn is_accepted(&self) -> bool {
        self.accepted_at.is_some()
    }

    /// Get invitation status
    pub fn status(&self) -> InvitationStatus {
        if self.is_accepted() {
            InvitationStatus::Accepted
        } else if self.is_expired() {
            InvitationStatus::Expired
        } else {
            InvitationStatus::Pending
        }
    }
}

/// Group with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithMetadata {
    #[serde(flatten)]
    pub group: AccessGroup,
    pub member_count: i32,
    pub user_role: Option<GroupRole>,
    pub is_owner: bool,
}

/// Member with user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWithUser {
    #[serde(flatten)]
    pub member: GroupMember,
    pub name: String,
    pub email: Option<String>,
}

/// Invitation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
}

/// Request to create a new group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to add a member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: GroupRole,
}

/// Request to update member role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: GroupRole,
}

/// Request to invite a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
    pub role: GroupRole,
}

/// Response for invitation acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationDetails {
    pub group_name: String,
    pub group_description: Option<String>,
    pub role: GroupRole,
    pub invited_by_name: String,
    pub created_at: String,
    pub expires_at: String,
}

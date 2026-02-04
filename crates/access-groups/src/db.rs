//! Database operations for access groups

use crate::error::{AccessGroupError, Result};
use crate::models::{
    AccessGroup, AddMemberRequest, CreateGroupRequest, GroupInvitation, GroupMember,
    GroupWithMetadata, InviteUserRequest, MemberWithUser, UpdateGroupRequest,
};
use chrono::{Duration, Utc};
use common::types::GroupRole;
use rand::Rng;
use sqlx::{Row, SqlitePool};

/// Generate a unique slug from a name
pub fn generate_slug(name: &str) -> String {
    slug::slugify(name)
}

/// Generate a secure random token for invitations
pub fn generate_invitation_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Create a new access group
pub async fn create_group(
    pool: &SqlitePool,
    owner_id: &str,
    request: CreateGroupRequest,
) -> Result<AccessGroup> {
    tracing::info!("Creating group: name={}, owner={}", request.name, owner_id);

    // Validate input
    if request.name.trim().is_empty() {
        return Err(AccessGroupError::InvalidInput(
            "Group name cannot be empty".to_string(),
        ));
    }

    // Generate slug
    let slug = generate_slug(&request.name);
    tracing::debug!("Generated slug: {}", slug);

    // Check if slug already exists
    let existing =
        sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM access_groups WHERE slug = ?")
            .bind(&slug)
            .fetch_one(pool)
            .await?;

    if existing > 0 {
        tracing::warn!("Slug already exists: {}", slug);
        return Err(AccessGroupError::SlugExists(slug));
    }

    // Create group
    tracing::debug!("Inserting group into database");
    let result = sqlx::query(
        r#"
        INSERT INTO access_groups (name, slug, description, owner_id)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&request.name)
    .bind(&slug)
    .bind(&request.description)
    .bind(owner_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert group: {:?}", e);
        e
    })?;

    let group_id = result.last_insert_rowid() as i32;
    tracing::info!("Group created with id: {}", group_id);

    // Add owner as member with owner role
    tracing::debug!("Adding owner as member");
    sqlx::query(
        r#"
        INSERT INTO group_members (group_id, user_id, role)
        VALUES (?, ?, 'owner')
        "#,
    )
    .bind(group_id)
    .bind(owner_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add owner as member: {:?}", e);
        e
    })?;

    // Fetch and return the created group
    tracing::debug!("Fetching created group");
    get_group_by_id(pool, group_id).await.map_err(|e| {
        tracing::error!("Failed to fetch created group: {:?}", e);
        e
    })
}

/// Get group by ID
pub async fn get_group_by_id(pool: &SqlitePool, id: i32) -> Result<AccessGroup> {
    sqlx::query_as::<_, AccessGroup>(
        r#"
        SELECT id, name, slug, description, owner_id, created_at, updated_at, is_active, settings
        FROM access_groups
        WHERE id = ? AND is_active = 1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AccessGroupError::GroupNotFound(id.to_string()))
}

/// Get group by slug
pub async fn get_group_by_slug(pool: &SqlitePool, slug: &str) -> Result<AccessGroup> {
    sqlx::query_as::<_, AccessGroup>(
        r#"
        SELECT id, name, slug, description, owner_id, created_at, updated_at, is_active, settings
        FROM access_groups
        WHERE slug = ? AND is_active = 1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AccessGroupError::GroupNotFound(slug.to_string()))
}

/// Get groups for a user with metadata
pub async fn get_user_groups(pool: &SqlitePool, user_id: &str) -> Result<Vec<GroupWithMetadata>> {
    let rows = sqlx::query(
        r#"
        SELECT
            g.id, g.name, g.slug, g.description, g.owner_id,
            g.created_at, g.updated_at, g.is_active, g.settings,
            COUNT(DISTINCT gm2.id) as member_count,
            gm.role as user_role
        FROM access_groups g
        INNER JOIN group_members gm ON g.id = gm.group_id
        LEFT JOIN group_members gm2 ON g.id = gm2.group_id
        WHERE gm.user_id = ? AND g.is_active = 1
        GROUP BY g.id, gm.role
        ORDER BY g.updated_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let groups = rows
        .into_iter()
        .map(|row| {
            let group = AccessGroup {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                owner_id: row.get("owner_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
                settings: row.get("settings"),
            };
            let member_count: i32 = row.get("member_count");
            let role_str: String = row.get("user_role");
            let user_role = role_str.parse::<GroupRole>().ok();
            let is_owner = group.owner_id == user_id;
            GroupWithMetadata {
                group,
                member_count,
                user_role,
                is_owner,
            }
        })
        .collect();

    Ok(groups)
}

/// Update group
pub async fn update_group(
    pool: &SqlitePool,
    slug: &str,
    request: UpdateGroupRequest,
) -> Result<AccessGroup> {
    let group = get_group_by_slug(pool, slug).await?;

    let name = request.name.unwrap_or(group.name);
    let description = request.description.or(group.description);

    // Generate new slug if name changed
    let new_slug = generate_slug(&name);

    // Check if new slug conflicts
    if new_slug != slug {
        let existing =
            sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM access_groups WHERE slug = ?")
                .bind(&new_slug)
                .fetch_one(pool)
                .await?;

        if existing > 0 {
            return Err(AccessGroupError::SlugExists(new_slug));
        }
    }

    sqlx::query(
        r#"
        UPDATE access_groups
        SET name = ?, slug = ?, description = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&name)
    .bind(&new_slug)
    .bind(&description)
    .bind(group.id)
    .execute(pool)
    .await?;

    get_group_by_slug(pool, &new_slug).await
}

/// Soft delete a group
pub async fn delete_group(pool: &SqlitePool, slug: &str) -> Result<()> {
    let group = get_group_by_slug(pool, slug).await?;

    sqlx::query("UPDATE access_groups SET is_active = 0 WHERE id = ?")
        .bind(group.id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Check if user is a member of a group
pub async fn is_group_member(pool: &SqlitePool, group_id: i32, user_id: &str) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

/// Get user's role in a group
pub async fn get_user_role(
    pool: &SqlitePool,
    group_id: i32,
    user_id: &str,
) -> Result<Option<GroupRole>> {
    let role = sqlx::query_scalar::<_, Option<String>>(
        "SELECT role FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match role {
        Some(Some(role_str)) => Ok(Some(
            role_str
                .parse()
                .map_err(|e: String| AccessGroupError::InvalidRole(e))?,
        )),
        _ => Ok(None),
    }
}

/// Check if user has permission to perform an action
pub async fn check_permission(
    pool: &SqlitePool,
    group_id: i32,
    user_id: &str,
    required_permission: &str,
) -> Result<bool> {
    let role = get_user_role(pool, group_id, user_id).await?;

    match role {
        Some(role) => {
            let has_permission = match required_permission {
                "read" => role.can_read(),
                "write" => role.can_write(),
                "delete" => role.can_delete(),
                "admin" => role.can_admin(),
                _ => false,
            };
            Ok(has_permission)
        }
        None => Ok(false),
    }
}

/// Get all members of a group
pub async fn get_group_members(pool: &SqlitePool, group_id: i32) -> Result<Vec<MemberWithUser>> {
    let rows = sqlx::query(
        r#"
        SELECT
            gm.id, gm.group_id, gm.user_id, gm.role, gm.joined_at, gm.invited_by,
            u.name, u.email
        FROM group_members gm
        INNER JOIN users u ON gm.user_id = u.id
        WHERE gm.group_id = ?
        ORDER BY
            CASE gm.role
                WHEN 'owner' THEN 1
                WHEN 'admin' THEN 2
                WHEN 'editor' THEN 3
                WHEN 'contributor' THEN 4
                WHEN 'viewer' THEN 5
            END,
            gm.joined_at ASC
        "#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    let members = rows
        .into_iter()
        .map(|row| {
            let member = GroupMember {
                id: row.get("id"),
                group_id: row.get("group_id"),
                user_id: row.get("user_id"),
                role: row.get("role"),
                joined_at: row.get("joined_at"),
                invited_by: row.get("invited_by"),
            };
            MemberWithUser {
                member,
                name: row.get("name"),
                email: row.get("email"),
            }
        })
        .collect();

    Ok(members)
}

/// Add a member to a group
pub async fn add_member(
    pool: &SqlitePool,
    group_id: i32,
    request: AddMemberRequest,
    invited_by: &str,
) -> Result<GroupMember> {
    // Check if user is already a member
    if is_group_member(pool, group_id, &request.user_id).await? {
        return Err(AccessGroupError::AlreadyMember);
    }

    // Validate role (cannot directly add as owner)
    if request.role == GroupRole::Owner {
        return Err(AccessGroupError::InvalidRole(
            "Cannot directly add a member as owner".to_string(),
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO group_members (group_id, user_id, role, invited_by)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(group_id)
    .bind(&request.user_id)
    .bind(request.role.to_string())
    .bind(invited_by)
    .execute(pool)
    .await?;

    // Fetch and return the member
    let member = sqlx::query_as::<_, GroupMember>(
        r#"
        SELECT id, group_id, user_id, role, joined_at, invited_by
        FROM group_members
        WHERE group_id = ? AND user_id = ?
        "#,
    )
    .bind(group_id)
    .bind(&request.user_id)
    .fetch_one(pool)
    .await?;

    Ok(member)
}

/// Remove a member from a group
pub async fn remove_member(pool: &SqlitePool, group_id: i32, user_id: &str) -> Result<()> {
    // Check if this is the last owner
    let owner_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND role = 'owner'",
    )
    .bind(group_id)
    .fetch_one(pool)
    .await?;

    let member_role = get_user_role(pool, group_id, user_id).await?;

    if member_role == Some(GroupRole::Owner) && owner_count <= 1 {
        return Err(AccessGroupError::CannotRemoveLastOwner);
    }

    let result = sqlx::query("DELETE FROM group_members WHERE group_id = ? AND user_id = ?")
        .bind(group_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AccessGroupError::MemberNotFound);
    }

    Ok(())
}

/// Update member role
pub async fn update_member_role(
    pool: &SqlitePool,
    group_id: i32,
    user_id: &str,
    new_role: GroupRole,
) -> Result<GroupMember> {
    // Check if trying to change owner role and they're the last owner
    let current_role = get_user_role(pool, group_id, user_id)
        .await?
        .ok_or(AccessGroupError::MemberNotFound)?;

    if current_role == GroupRole::Owner && new_role != GroupRole::Owner {
        let owner_count = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND role = 'owner'",
        )
        .bind(group_id)
        .fetch_one(pool)
        .await?;

        if owner_count <= 1 {
            return Err(AccessGroupError::CannotRemoveLastOwner);
        }
    }

    sqlx::query("UPDATE group_members SET role = ? WHERE group_id = ? AND user_id = ?")
        .bind(new_role.to_string())
        .bind(group_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Fetch and return updated member
    let member = sqlx::query_as::<_, GroupMember>(
        "SELECT id, group_id, user_id, role, joined_at, invited_by FROM group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(member)
}

/// Create an invitation
pub async fn create_invitation(
    pool: &SqlitePool,
    group_id: i32,
    request: InviteUserRequest,
    invited_by: &str,
) -> Result<GroupInvitation> {
    // Validate role (cannot invite as owner)
    if request.role == GroupRole::Owner {
        return Err(AccessGroupError::InvalidRole(
            "Cannot invite a user as owner".to_string(),
        ));
    }

    // Generate token
    let token = generate_invitation_token();

    // Set expiration (7 days from now)
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query(
        r#"
        INSERT INTO group_invitations (group_id, email, token, role, invited_by, expires_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(group_id)
    .bind(&request.email)
    .bind(&token)
    .bind(request.role.to_string())
    .bind(invited_by)
    .bind(expires_at.to_rfc3339())
    .execute(pool)
    .await?;

    // Fetch and return the invitation
    let invitation = sqlx::query_as::<_, GroupInvitation>(
        r#"
        SELECT id, group_id, email, token, role, invited_by, created_at, expires_at, accepted_at, accepted_by
        FROM group_invitations
        WHERE token = ?
        "#,
    )
    .bind(&token)
    .fetch_one(pool)
    .await?;

    Ok(invitation)
}

/// Get invitation by token
pub async fn get_invitation_by_token(pool: &SqlitePool, token: &str) -> Result<GroupInvitation> {
    sqlx::query_as::<_, GroupInvitation>(
        r#"
        SELECT id, group_id, email, token, role, invited_by, created_at, expires_at, accepted_at, accepted_by
        FROM group_invitations
        WHERE token = ?
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or(AccessGroupError::InvitationNotFound)
}

/// Get pending invitations for a group
pub async fn get_group_invitations(
    pool: &SqlitePool,
    group_id: i32,
) -> Result<Vec<GroupInvitation>> {
    let invitations = sqlx::query_as::<_, GroupInvitation>(
        r#"
        SELECT id, group_id, email, token, role, invited_by, created_at, expires_at, accepted_at, accepted_by
        FROM group_invitations
        WHERE group_id = ? AND accepted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    Ok(invitations)
}

/// Accept an invitation
pub async fn accept_invitation(
    pool: &SqlitePool,
    token: &str,
    user_id: &str,
) -> Result<GroupMember> {
    let invitation = get_invitation_by_token(pool, token).await?;

    // Check if already accepted
    if invitation.is_accepted() {
        return Err(AccessGroupError::InvitationAlreadyAccepted);
    }

    // Check if expired
    if invitation.is_expired() {
        return Err(AccessGroupError::InvitationExpired);
    }

    // Check if user is already a member
    if is_group_member(pool, invitation.group_id, user_id).await? {
        return Err(AccessGroupError::AlreadyMember);
    }

    // Add user as member
    let role_enum = invitation
        .role_enum()
        .map_err(|e| AccessGroupError::InvalidRole(e))?;
    let member = add_member(
        pool,
        invitation.group_id,
        AddMemberRequest {
            user_id: user_id.to_string(),
            role: role_enum,
        },
        &invitation.invited_by,
    )
    .await?;

    // Mark invitation as accepted
    sqlx::query(
        "UPDATE group_invitations SET accepted_at = CURRENT_TIMESTAMP, accepted_by = ? WHERE id = ?",
    )
    .bind(user_id)
    .bind(invitation.id)
    .execute(pool)
    .await?;

    Ok(member)
}

/// Cancel an invitation
pub async fn cancel_invitation(pool: &SqlitePool, invitation_id: i32) -> Result<()> {
    let result = sqlx::query("DELETE FROM group_invitations WHERE id = ?")
        .bind(invitation_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AccessGroupError::InvitationNotFound);
    }

    Ok(())
}

/// Get groups that contain a specific resource
pub async fn get_resource_groups(
    pool: &SqlitePool,
    resource_type: &str,
    resource_id: i32,
) -> Result<Vec<AccessGroup>> {
    let table = match resource_type {
        "video" => "videos",
        "image" => "images",
        "file" => "files",
        _ => {
            return Err(AccessGroupError::InvalidInput(
                "Invalid resource type".to_string(),
            ))
        }
    };

    let query = format!(
        r#"
        SELECT DISTINCT g.id, g.name, g.slug, g.description, g.owner_id,
               g.created_at, g.updated_at, g.is_active, g.settings
        FROM access_groups g
        INNER JOIN {} r ON g.id = r.group_id
        WHERE r.id = ? AND g.is_active = 1
        "#,
        table
    );

    let groups = sqlx::query_as::<_, AccessGroup>(&query)
        .bind(resource_id)
        .fetch_all(pool)
        .await?;

    Ok(groups)
}

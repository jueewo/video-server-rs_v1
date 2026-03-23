use crate::SqliteDatabase;
use db::access_groups::{
    AccessGroup, AccessGroupRepository, GroupInvitation, GroupMember, GroupWithMetadata,
    MemberWithUser,
};
use db::DbError;

fn map_err(e: sqlx::Error) -> DbError {
    match &e {
        sqlx::Error::Database(db_err) if db_err.message().contains("UNIQUE constraint") => {
            DbError::UniqueViolation(db_err.message().to_string())
        }
        _ => DbError::Internal(e.to_string()),
    }
}

#[derive(sqlx::FromRow)]
struct AccessGroupRow {
    id: i32,
    name: String,
    slug: String,
    description: Option<String>,
    owner_id: String,
    created_at: String,
    updated_at: String,
    is_active: i32,
    settings: Option<String>,
}

impl From<AccessGroupRow> for AccessGroup {
    fn from(r: AccessGroupRow) -> Self {
        AccessGroup {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            owner_id: r.owner_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            is_active: r.is_active,
            settings: r.settings,
        }
    }
}

#[derive(sqlx::FromRow)]
struct GroupMemberRow {
    id: i32,
    group_id: i32,
    user_id: String,
    role: String,
    joined_at: String,
    invited_by: Option<String>,
}

impl From<GroupMemberRow> for GroupMember {
    fn from(r: GroupMemberRow) -> Self {
        GroupMember {
            id: r.id,
            group_id: r.group_id,
            user_id: r.user_id,
            role: r.role,
            joined_at: r.joined_at,
            invited_by: r.invited_by,
        }
    }
}

#[derive(sqlx::FromRow)]
struct GroupInvitationRow {
    id: i32,
    group_id: i32,
    email: String,
    token: String,
    role: String,
    invited_by: String,
    created_at: String,
    expires_at: String,
    accepted_at: Option<String>,
    accepted_by: Option<String>,
}

impl From<GroupInvitationRow> for GroupInvitation {
    fn from(r: GroupInvitationRow) -> Self {
        GroupInvitation {
            id: r.id,
            group_id: r.group_id,
            email: r.email,
            token: r.token,
            role: r.role,
            invited_by: r.invited_by,
            created_at: r.created_at,
            expires_at: r.expires_at,
            accepted_at: r.accepted_at,
            accepted_by: r.accepted_by,
        }
    }
}

#[async_trait::async_trait]
impl AccessGroupRepository for SqliteDatabase {
    // ── Groups ──────────────────────────────────────────────────────

    async fn slug_exists(&self, slug: &str) -> Result<bool, DbError> {
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM access_groups WHERE slug = ?")
                .bind(slug)
                .fetch_one(self.pool())
                .await
                .map_err(map_err)?;
        Ok(count > 0)
    }

    async fn insert_group(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        owner_id: &str,
    ) -> Result<AccessGroup, DbError> {
        // Insert group
        sqlx::query(
            "INSERT INTO access_groups (name, slug, description, owner_id) VALUES (?, ?, ?, ?)",
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(owner_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        // Get the inserted group
        let group: AccessGroupRow = sqlx::query_as(
            "SELECT id, name, slug, description, owner_id, created_at, updated_at, is_active, settings \
             FROM access_groups WHERE slug = ?",
        )
        .bind(slug)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        // Add owner as member
        sqlx::query(
            "INSERT INTO group_members (group_id, user_id, role) VALUES (?, ?, 'owner')",
        )
        .bind(group.id)
        .bind(owner_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        Ok(group.into())
    }

    async fn get_group_by_id(&self, id: i32) -> Result<Option<AccessGroup>, DbError> {
        let row: Option<AccessGroupRow> = sqlx::query_as(
            "SELECT id, name, slug, description, owner_id, created_at, updated_at, is_active, settings \
             FROM access_groups WHERE id = ? AND is_active = 1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn get_group_by_slug(&self, slug: &str) -> Result<Option<AccessGroup>, DbError> {
        let row: Option<AccessGroupRow> = sqlx::query_as(
            "SELECT id, name, slug, description, owner_id, created_at, updated_at, is_active, settings \
             FROM access_groups WHERE slug = ? AND is_active = 1",
        )
        .bind(slug)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<GroupWithMetadata>, DbError> {
        // Complex JOIN query — parsed manually
        let rows: Vec<(i32, String, String, Option<String>, String, String, String, i32, Option<String>, i32, i64, String, i32)> = sqlx::query_as(
            "SELECT g.id, g.name, g.slug, g.description, g.owner_id, g.created_at, g.updated_at, \
                    g.is_active, g.settings, \
                    (SELECT COUNT(DISTINCT gm2.user_id) FROM group_members gm2 WHERE gm2.group_id = g.id) as member_count, \
                    (SELECT COUNT(*) FROM media_items mi WHERE mi.group_id = g.id) as media_count, \
                    gm.role, \
                    CASE WHEN g.owner_id = ? THEN 1 ELSE 0 END as is_owner \
             FROM access_groups g \
             INNER JOIN group_members gm ON g.id = gm.group_id AND gm.user_id = ? \
             WHERE g.is_active = 1 \
             ORDER BY g.created_at DESC",
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(id, name, slug, description, owner_id, created_at, updated_at, is_active, settings, member_count, media_count, role, is_owner)| {
                GroupWithMetadata {
                    group: AccessGroup {
                        id, name, slug, description, owner_id, created_at, updated_at, is_active, settings,
                    },
                    member_count,
                    media_count,
                    user_role: Some(role),
                    is_owner: is_owner != 0,
                }
            })
            .collect())
    }

    async fn update_group(
        &self,
        id: i32,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE access_groups SET name = ?, description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn soft_delete_group(&self, id: i32) -> Result<(), DbError> {
        sqlx::query("UPDATE access_groups SET is_active = 0 WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }

    // ── Members ─────────────────────────────────────────────────────

    async fn is_group_member(&self, group_id: i32, user_id: &str) -> Result<bool, DbError> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;
        Ok(count > 0)
    }

    async fn get_user_role(
        &self,
        group_id: i32,
        user_id: &str,
    ) -> Result<Option<String>, DbError> {
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM group_members WHERE group_id = ? AND user_id = ?",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(role)
    }

    async fn get_group_members(&self, group_id: i32) -> Result<Vec<MemberWithUser>, DbError> {
        let rows: Vec<(i32, i32, String, String, String, Option<String>, String, Option<String>)> = sqlx::query_as(
            "SELECT gm.id, gm.group_id, gm.user_id, gm.role, gm.joined_at, gm.invited_by, \
                    u.name, u.email \
             FROM group_members gm \
             INNER JOIN users u ON gm.user_id = u.id \
             WHERE gm.group_id = ? \
             ORDER BY CASE gm.role \
                WHEN 'owner' THEN 1 WHEN 'admin' THEN 2 WHEN 'editor' THEN 3 \
                WHEN 'contributor' THEN 4 WHEN 'viewer' THEN 5 ELSE 6 END, \
                gm.joined_at ASC",
        )
        .bind(group_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|(id, group_id, user_id, role, joined_at, invited_by, name, email)| MemberWithUser {
                member: GroupMember { id, group_id, user_id, role, joined_at, invited_by },
                name,
                email,
            })
            .collect())
    }

    async fn add_member(
        &self,
        group_id: i32,
        user_id: &str,
        role: &str,
        invited_by: Option<&str>,
    ) -> Result<GroupMember, DbError> {
        sqlx::query(
            "INSERT INTO group_members (group_id, user_id, role, invited_by) VALUES (?, ?, ?, ?)",
        )
        .bind(group_id)
        .bind(user_id)
        .bind(role)
        .bind(invited_by)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        let member: GroupMemberRow = sqlx::query_as(
            "SELECT id, group_id, user_id, role, joined_at, invited_by \
             FROM group_members WHERE group_id = ? AND user_id = ?",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        Ok(member.into())
    }

    async fn remove_member(&self, group_id: i32, user_id: &str) -> Result<bool, DbError> {
        let result =
            sqlx::query("DELETE FROM group_members WHERE group_id = ? AND user_id = ?")
                .bind(group_id)
                .bind(user_id)
                .execute(self.pool())
                .await
                .map_err(map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn update_member_role(
        &self,
        group_id: i32,
        user_id: &str,
        role: &str,
    ) -> Result<Option<GroupMember>, DbError> {
        sqlx::query("UPDATE group_members SET role = ? WHERE group_id = ? AND user_id = ?")
            .bind(role)
            .bind(group_id)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;

        let member: Option<GroupMemberRow> = sqlx::query_as(
            "SELECT id, group_id, user_id, role, joined_at, invited_by \
             FROM group_members WHERE group_id = ? AND user_id = ?",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;

        Ok(member.map(Into::into))
    }

    async fn count_owners(&self, group_id: i32) -> Result<i32, DbError> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND role = 'owner'",
        )
        .bind(group_id)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;
        Ok(count)
    }

    // ── Invitations ─────────────────────────────────────────────────

    async fn create_invitation(
        &self,
        group_id: i32,
        email: &str,
        token: &str,
        role: &str,
        invited_by: &str,
        expires_at: &str,
    ) -> Result<GroupInvitation, DbError> {
        sqlx::query(
            "INSERT INTO group_invitations (group_id, email, token, role, invited_by, expires_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(group_id)
        .bind(email)
        .bind(token)
        .bind(role)
        .bind(invited_by)
        .bind(expires_at)
        .execute(self.pool())
        .await
        .map_err(map_err)?;

        let invitation: GroupInvitationRow = sqlx::query_as(
            "SELECT id, group_id, email, token, role, invited_by, created_at, expires_at, accepted_at, accepted_by \
             FROM group_invitations WHERE token = ?",
        )
        .bind(token)
        .fetch_one(self.pool())
        .await
        .map_err(map_err)?;

        Ok(invitation.into())
    }

    async fn get_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<Option<GroupInvitation>, DbError> {
        let row: Option<GroupInvitationRow> = sqlx::query_as(
            "SELECT id, group_id, email, token, role, invited_by, created_at, expires_at, accepted_at, accepted_by \
             FROM group_invitations WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(self.pool())
        .await
        .map_err(map_err)?;
        Ok(row.map(Into::into))
    }

    async fn get_group_invitations(
        &self,
        group_id: i32,
    ) -> Result<Vec<GroupInvitation>, DbError> {
        let rows: Vec<GroupInvitationRow> = sqlx::query_as(
            "SELECT id, group_id, email, token, role, invited_by, created_at, expires_at, accepted_at, accepted_by \
             FROM group_invitations WHERE group_id = ? AND accepted_at IS NULL \
             ORDER BY created_at DESC",
        )
        .bind(group_id)
        .fetch_all(self.pool())
        .await
        .map_err(map_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn mark_invitation_accepted(
        &self,
        invitation_id: i32,
        user_id: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE group_invitations SET accepted_at = CURRENT_TIMESTAMP, accepted_by = ? WHERE id = ?",
        )
        .bind(user_id)
        .bind(invitation_id)
        .execute(self.pool())
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete_invitation(&self, invitation_id: i32) -> Result<(), DbError> {
        sqlx::query("DELETE FROM group_invitations WHERE id = ?")
            .bind(invitation_id)
            .execute(self.pool())
            .await
            .map_err(map_err)?;
        Ok(())
    }
}

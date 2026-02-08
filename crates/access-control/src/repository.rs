//! Type-safe database repository for access control queries
//!
//! This module provides a clean, type-safe interface to the database,
//! eliminating SQL injection risks by avoiding string concatenation
//! and using explicit match statements for each resource type.

use crate::{AccessError, AccessKeyData, Permission};
use common::{GroupRole, ResourceType};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Type-safe database repository for access control
///
/// All queries are written explicitly for each resource type,
/// avoiding the SQL injection risks of dynamic table name construction.
pub struct AccessRepository {
    pool: SqlitePool,
}

impl AccessRepository {
    /// Create a new repository instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Check if a resource is marked as public
    ///
    /// # Type Safety
    ///
    /// Each resource type has its own explicit query - no string concatenation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::{AccessRepository, Permission};
    /// # use common::ResourceType;
    /// # async fn example(repo: AccessRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let is_public = repo.is_resource_public(ResourceType::Video, 42).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_resource_public(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        match resource_type {
            ResourceType::Video => {
                let result: Option<bool> =
                    sqlx::query_scalar("SELECT is_public FROM videos WHERE id = ?")
                        .bind(resource_id)
                        .fetch_optional(&self.pool)
                        .await?;
                result.ok_or(AccessError::NotFound {
                    resource_type: "Video".to_string(),
                    resource_id,
                })
            }
            ResourceType::Image => {
                let result: Option<bool> =
                    sqlx::query_scalar("SELECT is_public FROM images WHERE id = ?")
                        .bind(resource_id)
                        .fetch_optional(&self.pool)
                        .await?;
                result.ok_or(AccessError::NotFound {
                    resource_type: "Image".to_string(),
                    resource_id,
                })
            }
            ResourceType::File => {
                let result: Option<bool> =
                    sqlx::query_scalar("SELECT is_public FROM files WHERE id = ?")
                        .bind(resource_id)
                        .fetch_optional(&self.pool)
                        .await?;
                result.ok_or(AccessError::NotFound {
                    resource_type: "File".to_string(),
                    resource_id,
                })
            }
            ResourceType::Folder => {
                let result: Option<bool> =
                    sqlx::query_scalar("SELECT is_public FROM folders WHERE id = ?")
                        .bind(resource_id)
                        .fetch_optional(&self.pool)
                        .await?;
                result.ok_or(AccessError::NotFound {
                    resource_type: "Folder".to_string(),
                    resource_id,
                })
            }
        }
    }

    /// Check if a user owns a specific resource
    ///
    /// # Type Safety
    ///
    /// Explicit queries for each resource type prevent SQL injection.
    pub async fn is_resource_owner(
        &self,
        user_id: &str,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        let owner: Option<String> = match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT user_id FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT user_id FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT user_id FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT user_id FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
        };

        Ok(owner.as_deref() == Some(user_id))
    }

    /// Get the group ID associated with a resource
    ///
    /// Returns None if the resource doesn't belong to any group.
    pub async fn get_resource_group(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<Option<i32>, AccessError> {
        match resource_type {
            ResourceType::Video => sqlx::query_scalar("SELECT group_id FROM videos WHERE id = ?")
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(Into::into),
            ResourceType::Image => sqlx::query_scalar("SELECT group_id FROM images WHERE id = ?")
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(Into::into),
            ResourceType::File => sqlx::query_scalar("SELECT group_id FROM files WHERE id = ?")
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(Into::into),
            ResourceType::Folder => sqlx::query_scalar("SELECT group_id FROM folders WHERE id = ?")
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(Into::into),
        }
    }

    /// Get a user's role in a specific group
    ///
    /// Returns None if the user is not a member of the group.
    pub async fn get_user_group_role(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<GroupRole>, AccessError> {
        let role: Option<String> =
            sqlx::query_scalar("SELECT role FROM group_members WHERE user_id = ? AND group_id = ?")
                .bind(user_id)
                .bind(group_id)
                .fetch_optional(&self.pool)
                .await?;

        match role {
            Some(r) => GroupRole::from_str(&r)
                .map(Some)
                .map_err(|e| AccessError::Internal { message: e }),
            None => Ok(None),
        }
    }

    /// Get access key data from the database
    ///
    /// Returns None if the key doesn't exist or is inactive.
    pub async fn get_access_key_data(
        &self,
        key: &str,
    ) -> Result<Option<AccessKeyData>, AccessError> {
        let result: Option<(
            i32,
            String,
            String,
            String,
            Option<i32>,
            bool,
            Option<String>,
            Option<i32>,
            i32,
            bool,
        )> = sqlx::query_as(
            "SELECT
                id,
                code as key,
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
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((
                id,
                key,
                description,
                permission_str,
                access_group_id,
                share_all_group_resources,
                expires_at,
                max_downloads,
                current_downloads,
                is_active,
            )) => {
                let permission_level = Permission::from_str(&permission_str).map_err(|_| {
                    AccessError::InvalidPermission {
                        value: permission_str.clone(),
                    }
                })?;

                Ok(Some(AccessKeyData {
                    id,
                    key,
                    description,
                    permission_level,
                    access_group_id,
                    share_all_group_resources,
                    expires_at,
                    max_downloads,
                    current_downloads,
                    is_active,
                }))
            }
            None => Ok(None),
        }
    }

    /// Check if an access key grants access to a specific resource
    ///
    /// Handles both group-wide keys and individual resource keys.
    pub async fn access_key_grants_resource(
        &self,
        key_data: &AccessKeyData,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        // If this is a group-wide key, check if resource belongs to the group
        if key_data.share_all_group_resources {
            if let Some(group_id) = key_data.access_group_id {
                let resource_group = self.get_resource_group(resource_type, resource_id).await?;
                return Ok(resource_group == Some(group_id));
            }
        }

        // Otherwise, check if key has explicit permission for this resource
        let has_permission: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM access_key_permissions
                WHERE access_key_id = ?
                  AND resource_type = ?
                  AND resource_id = ?
            )",
        )
        .bind(key_data.id)
        .bind(resource_type.to_string())
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(has_permission)
    }

    /// Increment download count for an access key
    ///
    /// Used when a download actually occurs.
    pub async fn increment_download_count(&self, key: &str) -> Result<(), AccessError> {
        sqlx::query(
            "UPDATE access_codes
             SET current_downloads = current_downloads + 1,
                 last_accessed_at = CURRENT_TIMESTAMP
             WHERE code = ?",
        )
        .bind(key)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a resource exists
    pub async fn resource_exists(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        let exists = match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM videos WHERE id = ?)")
                    .bind(resource_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM images WHERE id = ?)")
                    .bind(resource_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE id = ?)")
                    .bind(resource_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM folders WHERE id = ?)")
                    .bind(resource_id)
                    .fetch_one(&self.pool)
                    .await?
            }
        };

        Ok(exists)
    }

    /// Get resource title/name for better error messages
    pub async fn get_resource_title(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<String, AccessError> {
        let title: Option<String> = match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT title FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT title FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT title FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT name FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
        };

        title.ok_or(AccessError::NotFound {
            resource_type: resource_type.to_string(),
            resource_id,
        })
    }

    /// Check if a user is a member of any group
    pub async fn is_user_in_group(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<bool, AccessError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM group_members
                WHERE user_id = ? AND group_id = ?
            )",
        )
        .bind(user_id)
        .bind(group_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Get all group IDs a user is a member of
    pub async fn get_user_groups(&self, user_id: &str) -> Result<Vec<i32>, AccessError> {
        let groups: Vec<i32> = sqlx::query_scalar(
            "SELECT group_id FROM group_members WHERE user_id = ? ORDER BY group_id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    /// Batch check if multiple resources are public
    ///
    /// Returns a vector of (resource_id, is_public) tuples.
    /// More efficient than checking one by one.
    pub async fn batch_check_public(
        &self,
        resource_type: ResourceType,
        resource_ids: &[i32],
    ) -> Result<Vec<(i32, bool)>, AccessError> {
        if resource_ids.is_empty() {
            return Ok(vec![]);
        }

        // Build placeholders for IN clause
        let placeholders = resource_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");

        let results: Vec<(i32, bool)> = match resource_type {
            ResourceType::Video => {
                let query = format!(
                    "SELECT id, is_public FROM videos WHERE id IN ({})",
                    placeholders
                );
                let mut q = sqlx::query_as(&query);
                for id in resource_ids {
                    q = q.bind(id);
                }
                q.fetch_all(&self.pool).await?
            }
            ResourceType::Image => {
                let query = format!(
                    "SELECT id, is_public FROM images WHERE id IN ({})",
                    placeholders
                );
                let mut q = sqlx::query_as(&query);
                for id in resource_ids {
                    q = q.bind(id);
                }
                q.fetch_all(&self.pool).await?
            }
            ResourceType::File => {
                let query = format!(
                    "SELECT id, is_public FROM files WHERE id IN ({})",
                    placeholders
                );
                let mut q = sqlx::query_as(&query);
                for id in resource_ids {
                    q = q.bind(id);
                }
                q.fetch_all(&self.pool).await?
            }
            ResourceType::Folder => {
                let query = format!(
                    "SELECT id, is_public FROM folders WHERE id IN ({})",
                    placeholders
                );
                let mut q = sqlx::query_as(&query);
                for id in resource_ids {
                    q = q.bind(id);
                }
                q.fetch_all(&self.pool).await?
            }
        };

        Ok(results)
    }

    /// Get the owner ID of a resource
    pub async fn get_resource_owner(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<Option<String>, AccessError> {
        let owner: Option<String> = match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT user_id FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT user_id FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT user_id FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT user_id FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
        };

        Ok(owner)
    }

    /// Get visibility status of a resource
    pub async fn get_resource_visibility(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<String, AccessError> {
        let visibility: Option<String> = match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT visibility FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT visibility FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT visibility FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT visibility FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
        };

        visibility.ok_or(AccessError::NotFound {
            resource_type: resource_type.to_string(),
            resource_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE videos (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                user_id TEXT NOT NULL,
                group_id INTEGER,
                is_public BOOLEAN NOT NULL DEFAULT 0,
                visibility TEXT NOT NULL DEFAULT 'private'
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE images (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                user_id TEXT NOT NULL,
                group_id INTEGER,
                is_public BOOLEAN NOT NULL DEFAULT 0,
                visibility TEXT NOT NULL DEFAULT 'private'
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE group_members (
                id INTEGER PRIMARY KEY,
                user_id TEXT NOT NULL,
                group_id INTEGER NOT NULL,
                role TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE access_codes (
                id INTEGER PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                permission_level TEXT NOT NULL DEFAULT 'read',
                access_group_id INTEGER,
                share_all_group_resources BOOLEAN NOT NULL DEFAULT 0,
                expires_at TEXT,
                max_downloads INTEGER,
                current_downloads INTEGER NOT NULL DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                last_accessed_at TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_is_resource_public() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());

        // Insert test video
        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Test Video")
            .bind("user123")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();

        let is_public = repo
            .is_resource_public(ResourceType::Video, 1)
            .await
            .unwrap();
        assert!(is_public);
    }

    #[tokio::test]
    async fn test_is_resource_owner() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());

        sqlx::query("INSERT INTO videos (id, title, user_id, is_public) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Test Video")
            .bind("user123")
            .bind(false)
            .execute(&pool)
            .await
            .unwrap();

        let is_owner = repo
            .is_resource_owner("user123", ResourceType::Video, 1)
            .await
            .unwrap();
        assert!(is_owner);

        let is_owner = repo
            .is_resource_owner("other_user", ResourceType::Video, 1)
            .await
            .unwrap();
        assert!(!is_owner);
    }

    #[tokio::test]
    async fn test_get_resource_group() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());

        sqlx::query(
            "INSERT INTO videos (id, title, user_id, group_id, is_public) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("Test Video")
        .bind("user123")
        .bind(5)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        let group_id = repo
            .get_resource_group(ResourceType::Video, 1)
            .await
            .unwrap();
        assert_eq!(group_id, Some(5));
    }

    #[tokio::test]
    async fn test_get_user_group_role() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());

        sqlx::query("INSERT INTO group_members (id, user_id, group_id, role) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("user123")
            .bind(5)
            .bind("editor")
            .execute(&pool)
            .await
            .unwrap();

        let role = repo.get_user_group_role("user123", 5).await.unwrap();
        assert_eq!(role, Some(GroupRole::Editor));

        let role = repo.get_user_group_role("other_user", 5).await.unwrap();
        assert_eq!(role, None);
    }

    #[tokio::test]
    async fn test_get_access_key_data() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool.clone());

        sqlx::query(
            "INSERT INTO access_codes
             (id, code, description, permission_level, is_active)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1)
        .bind("test-key")
        .bind("Test Key")
        .bind("download")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        let key_data = repo.get_access_key_data("test-key").await.unwrap();
        assert!(key_data.is_some());
        let key_data = key_data.unwrap();
        assert_eq!(key_data.key, "test-key");
        assert_eq!(key_data.permission_level, Permission::Download);
    }

    #[tokio::test]
    async fn test_resource_not_found() {
        let pool = setup_test_db().await;
        let repo = AccessRepository::new(pool);

        let result = repo.is_resource_public(ResourceType::Video, 999).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AccessError::NotFound { .. }));
    }
}

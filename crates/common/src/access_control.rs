use crate::{Error, ResourceType};
use sqlx::SqlitePool;
use tracing::warn;

/// Check if a user or access key has access to a resource
/// This implements the 4-layer access control model:
/// 1. Public - anyone can access
/// 2. Private (Owner only) - requires authentication
/// 3. Access Group - requires group membership
/// 4. Access Key - anonymous access with key
pub async fn check_resource_access(
    pool: &SqlitePool,
    user_id: Option<&str>,
    access_key: Option<&str>,
    resource_type: ResourceType,
    resource_id: i32,
) -> Result<bool, Error> {
    // Layer 1: Check if resource is public
    if is_public(pool, resource_type.clone(), resource_id).await? {
        return Ok(true);
    }

    // Layer 2: Check access key (if provided)
    if let Some(key) = access_key {
        return check_access_key_permission(pool, key, resource_type, resource_id).await;
    }

    // Layers 3 & 4 require authentication
    let user = user_id.ok_or(Error::Unauthorized)?;

    // Layer 3: Check ownership
    if is_owner(pool, user, resource_type.clone(), resource_id).await? {
        return Ok(true);
    }

    // Layer 4: Check group membership
    check_group_membership_access(pool, user, resource_type, resource_id).await
}

async fn is_public(
    pool: &SqlitePool,
    resource_type: ResourceType,
    resource_id: i32,
) -> Result<bool, Error> {
    let table = match resource_type {
        ResourceType::Video => "videos",
        ResourceType::Image => "images",
        ResourceType::File => "files",
        ResourceType::Folder => "folders",
    };

    let query = format!("SELECT is_public FROM {} WHERE id = ?", table);
    let is_public: bool = sqlx::query_scalar(&query)
        .bind(resource_id)
        .fetch_optional(pool)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(is_public)
}

async fn is_owner(
    pool: &SqlitePool,
    user_id: &str,
    resource_type: ResourceType,
    resource_id: i32,
) -> Result<bool, Error> {
    let table = match resource_type {
        ResourceType::Video => "videos",
        ResourceType::Image => "images",
        ResourceType::File => "files",
        ResourceType::Folder => "folders",
    };

    let query = format!("SELECT user_id FROM {} WHERE id = ?", table);
    let owner: Option<String> = sqlx::query_scalar(&query)
        .bind(resource_id)
        .fetch_optional(pool)
        .await?;

    Ok(owner.as_deref() == Some(user_id))
}

async fn check_group_membership_access(
    pool: &SqlitePool,
    user_id: &str,
    resource_type: ResourceType,
    resource_id: i32,
) -> Result<bool, Error> {
    // Get the group_id for this resource
    let table = match resource_type {
        ResourceType::Video => "videos",
        ResourceType::Image => "images",
        ResourceType::File => "files",
        ResourceType::Folder => "folders",
    };

    let query = format!("SELECT group_id FROM {} WHERE id = ?", table);
    let group_id: Option<Option<i32>> = sqlx::query_scalar(&query)
        .bind(resource_id)
        .fetch_optional(pool)
        .await?;

    let group_id = match group_id.flatten() {
        Some(id) => id,
        None => return Ok(false), // No group associated
    };

    // Check if user is a member of the group
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = ? AND user_id = ?)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(is_member)
}

async fn check_access_key_permission(
    pool: &SqlitePool,
    key: &str,
    resource_type: ResourceType,
    resource_id: i32,
) -> Result<bool, Error> {
    // Get access key details
    let key_record: Option<(i32, Option<i32>, bool, Option<String>, Option<i32>, i32)> =
        sqlx::query_as(
            "SELECT id, access_group_id, share_all_group_resources, expires_at, max_downloads, current_downloads
             FROM access_keys WHERE key = ? AND is_active = 1",
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;

    let (key_id, group_id, share_all, expires_at, max_downloads, current_downloads) =
        key_record.ok_or(Error::NotFound)?;

    // Check expiration
    if let Some(exp) = expires_at {
        let expires = time::OffsetDateTime::parse(
            &exp,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .map_err(|_| Error::Internal("Invalid expiration date".to_string()))?;

        if expires < time::OffsetDateTime::now_utc() {
            return Err(Error::ExpiredKey);
        }
    }

    // Check download limit
    if let Some(max) = max_downloads {
        if current_downloads >= max {
            return Err(Error::DownloadLimitExceeded);
        }
    }

    // Check permission
    if share_all {
        // Check if resource belongs to the group
        if let Some(gid) = group_id {
            check_resource_in_group(pool, resource_type, resource_id, gid).await
        } else {
            Ok(false)
        }
    } else {
        // Check specific permission
        let has_permission: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM access_key_permissions
             WHERE access_key_id = ? AND resource_type = ? AND resource_id = ?)",
        )
        .bind(key_id)
        .bind(resource_type.to_string())
        .bind(resource_id)
        .fetch_one(pool)
        .await?;

        Ok(has_permission)
    }
}

async fn check_resource_in_group(
    pool: &SqlitePool,
    resource_type: ResourceType,
    resource_id: i32,
    group_id: i32,
) -> Result<bool, Error> {
    let table = match resource_type {
        ResourceType::Video => "videos",
        ResourceType::Image => "images",
        ResourceType::File => "files",
        ResourceType::Folder => "folders",
    };

    let query = format!("SELECT group_id FROM {} WHERE id = ?", table);
    let res_group: Option<Option<i32>> = sqlx::query_scalar(&query)
        .bind(resource_id)
        .fetch_optional(pool)
        .await?;

    Ok(res_group.flatten() == Some(group_id))
}

/// Log access key usage for analytics
pub async fn log_access_key_usage(
    pool: &SqlitePool,
    key: &str,
    resource_type: Option<ResourceType>,
    resource_id: Option<i32>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    referer: Option<&str>,
) -> Result<(), Error> {
    // Get key ID
    let key_id: Option<i32> = sqlx::query_scalar("SELECT id FROM access_keys WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    let key_id = match key_id {
        Some(id) => id,
        None => {
            warn!("Attempted to log usage for non-existent key: {}", key);
            return Ok(()); // Don't fail, just warn
        }
    };

    // Insert log
    sqlx::query(
        "INSERT INTO access_key_logs (access_key_id, resource_type, resource_id, ip_address, user_agent, referer)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(key_id)
    .bind(resource_type.map(|t| t.to_string()))
    .bind(resource_id)
    .bind(ip_address)
    .bind(user_agent)
    .bind(referer)
    .execute(pool)
    .await?;

    // Update access count and last accessed
    sqlx::query(
        "UPDATE access_keys
         SET access_count = access_count + 1,
             last_accessed_at = CURRENT_TIMESTAMP,
             last_accessed_ip = ?
         WHERE id = ?",
    )
    .bind(ip_address)
    .bind(key_id)
    .execute(pool)
    .await?;

    Ok(())
}

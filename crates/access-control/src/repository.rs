//! Type-safe database repository for access control queries
//!
//! This module provides a clean, type-safe interface to the database,
//! delegating to the `db::access_control::AccessControlRepository` trait.

use crate::{AccessError, AccessKeyData, Permission};
use common::{GroupRole, ResourceType};
use db::access_control::AccessControlRepository;
use std::str::FromStr;
use std::sync::Arc;

/// Helper: convert ResourceType to the string used by the DB trait.
fn resource_type_str(rt: ResourceType) -> &'static str {
    match rt {
        ResourceType::Video => "video",
        ResourceType::Image => "image",
        ResourceType::File => "document",
        ResourceType::Folder => "folder",
    }
}

/// Type-safe database repository for access control.
///
/// Wraps a `dyn AccessControlRepository` and translates between
/// domain types (`ResourceType`, `Permission`, `AccessKeyData`) and
/// the raw types returned by the DB layer.
pub struct AccessRepository {
    repo: Arc<dyn AccessControlRepository>,
}

impl AccessRepository {
    /// Create a new repository instance.
    pub fn new(repo: Arc<dyn AccessControlRepository>) -> Self {
        Self { repo }
    }

    /// Check if a resource is marked as public.
    pub async fn is_resource_public(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        let result = self
            .repo
            .is_resource_public(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        result.ok_or(AccessError::NotFound {
            resource_type: resource_type.to_string(),
            resource_id,
        })
    }

    /// Check if a user owns a specific resource.
    pub async fn is_resource_owner(
        &self,
        user_id: &str,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        let owner = self
            .repo
            .get_resource_owner(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        Ok(owner.as_deref() == Some(user_id))
    }

    /// Get the group ID associated with a resource.
    pub async fn get_resource_group(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<Option<i32>, AccessError> {
        let result = self
            .repo
            .get_resource_group(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        // None = resource not found (return None as "no group"),
        // Some(inner) = resource found, inner is the group_id
        Ok(result.flatten())
    }

    /// Get a user's role in a specific group.
    pub async fn get_user_group_role(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<GroupRole>, AccessError> {
        let role = self
            .repo
            .get_user_group_role(user_id, group_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        match role {
            Some(r) => GroupRole::from_str(&r)
                .map(Some)
                .map_err(|e| AccessError::Internal { message: e }),
            None => Ok(None),
        }
    }

    /// Get access key data from the database.
    pub async fn get_access_key_data(
        &self,
        key: &str,
    ) -> Result<Option<AccessKeyData>, AccessError> {
        let row = self
            .repo
            .get_access_key_data(key)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        match row {
            Some(r) => {
                let permission_level =
                    Permission::from_str(&r.permission_level).map_err(|_| {
                        AccessError::InvalidPermission {
                            value: r.permission_level.clone(),
                        }
                    })?;

                Ok(Some(AccessKeyData {
                    id: r.id,
                    key: r.key,
                    description: r.description,
                    permission_level,
                    access_group_id: r.access_group_id,
                    share_all_group_resources: r.share_all_group_resources,
                    expires_at: r.expires_at,
                    max_downloads: r.max_downloads,
                    current_downloads: r.current_downloads,
                    is_active: r.is_active,
                }))
            }
            None => Ok(None),
        }
    }

    /// Check if an access key grants access to a specific resource.
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

        let rt_str = resource_type_str(resource_type);

        // Get slug for this resource
        let slug = self
            .repo
            .get_resource_slug(resource_id, rt_str)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        let Some(slug) = slug else {
            return Ok(false);
        };

        // Check if key has permission for this resource
        self.repo
            .access_code_has_permission(key_data.id, rt_str, &slug)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Increment download count for an access key.
    pub async fn increment_download_count(&self, key: &str) -> Result<(), AccessError> {
        self.repo
            .increment_download_count(key)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Check if a resource exists.
    pub async fn resource_exists(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, AccessError> {
        self.repo
            .resource_exists(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Get resource title/name.
    pub async fn get_resource_title(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<String, AccessError> {
        let title = self
            .repo
            .get_resource_title(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        title.ok_or(AccessError::NotFound {
            resource_type: resource_type.to_string(),
            resource_id,
        })
    }

    /// Check if a user is a member of any group.
    pub async fn is_user_in_group(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<bool, AccessError> {
        self.repo
            .is_user_in_group(user_id, group_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Get all group IDs a user is a member of.
    pub async fn get_user_groups(&self, user_id: &str) -> Result<Vec<i32>, AccessError> {
        self.repo
            .get_user_groups(user_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Batch check if multiple resources are public.
    pub async fn batch_check_public(
        &self,
        resource_type: ResourceType,
        resource_ids: &[i32],
    ) -> Result<Vec<(i32, bool)>, AccessError> {
        self.repo
            .batch_check_public(resource_type_str(resource_type), resource_ids)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Get the owner ID of a resource.
    pub async fn get_resource_owner(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<Option<String>, AccessError> {
        self.repo
            .get_resource_owner(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Get visibility status of a resource.
    pub async fn get_resource_visibility(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<String, AccessError> {
        let visibility = self
            .repo
            .get_resource_visibility(resource_type_str(resource_type), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        visibility.ok_or(AccessError::NotFound {
            resource_type: resource_type.to_string(),
            resource_id,
        })
    }
}

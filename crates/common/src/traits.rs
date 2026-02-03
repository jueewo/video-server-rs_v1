use crate::{Error, Permission, ResourceType};
use async_trait::async_trait;

/// Access control trait that all resource managers should implement
#[async_trait]
pub trait AccessControl {
    /// Check if a user has permission to access a resource
    async fn check_access(
        &self,
        user_id: &str,
        resource_type: ResourceType,
        resource_id: i32,
        permission: Permission,
    ) -> Result<bool, Error>;

    /// Check if a user has access via group membership
    async fn check_group_access(
        &self,
        user_id: &str,
        group_id: i32,
        permission: Permission,
    ) -> Result<bool, Error>;

    /// Check if an access key grants permission to a resource
    async fn check_key_access(
        &self,
        key: &str,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, Error>;
}

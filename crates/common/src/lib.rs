//! Common types, traits, and utilities shared across all crates

pub mod access_control;
pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use access_control::{check_resource_access, log_access_key_usage};
pub use error::Error;
pub use traits::AccessControl;
pub use types::{GroupRole, Permission, ResourceMetadata, ResourceType};

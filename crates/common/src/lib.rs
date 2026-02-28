//! Common types, traits, and utilities shared across all crates

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod request_id;
pub mod routes;
pub mod services;
pub mod storage;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use error::{ApiError, Error};
pub use models::*;
pub use storage::{MediaType, UserStorageManager};
pub use traits::AccessControl;
pub use types::{GroupRole, Permission, ResourceMetadata, ResourceType};
pub use utils::*;

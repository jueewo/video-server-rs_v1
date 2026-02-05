//! Common types, traits, and utilities shared across all crates

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod services;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use db::*;
pub use error::Error;
pub use handlers::*;
pub use models::*;
pub use routes::*;
pub use services::*;
pub use traits::AccessControl;
pub use types::{GroupRole, Permission, ResourceMetadata, ResourceType};
pub use utils::*;

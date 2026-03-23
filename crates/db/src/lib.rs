//! Database domain traits.
//!
//! This crate defines repository traits for each domain.
//! It has **no** dependency on any database driver (sqlx, diesel, etc.).
//! Implementations live in `db-sqlite` (or a future `db-postgres`).

pub mod access_codes;
pub mod access_control;
pub mod access_groups;
pub mod agents;
pub mod api_keys;
pub mod error;
pub mod federation;
pub mod git_providers;
pub mod llm_providers;
pub mod media;
pub mod publications;
pub mod user_auth;
pub mod vaults;
pub mod workspaces;

pub use error::DbError;

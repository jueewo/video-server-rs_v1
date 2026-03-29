//! SQLite implementations of the `db` repository traits.
//!
//! The single [`SqliteDatabase`] struct implements all domain repository traits.
//! Pass it as `Arc<dyn AgentRepository>` (or other trait) into your state.

pub mod access_codes;
pub mod access_control;
pub mod access_groups;
pub mod agents;
pub mod api_keys;
pub mod federation;
pub mod git_providers;
pub mod llm_providers;
pub mod media;
pub mod processes;
pub mod publications;
pub mod schedules;
pub mod user_auth;
pub mod vaults;
pub mod workspaces;

use sqlx::SqlitePool;

/// Shared database handle. Implements all domain repository traits.
#[derive(Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

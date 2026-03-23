//! Legacy compatibility — delegates to the `AgentRepository` trait.
//!
//! This module is kept so that internal callers (`lib.rs`) can continue
//! calling `db::list_user_agents(repo, ...)` etc. during incremental migration.
//! New code should use the trait directly.

pub use db::agents::AgentRepository;

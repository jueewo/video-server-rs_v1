//! Media Hub - Unified Media Management UI
//!
//! Provides a unified interface for managing videos, images, and documents
//! through a single UI with cross-media search, filtering, and operations.
//!
//! ## Features
//!
//! - Unified media list view (all types in one view)
//! - Cross-media search and filtering
//! - Generic upload form (auto-detects type)
//! - Unified media cards with type indicators
//! - Responsive design
//! - Accessibility support
//!
//! ## Usage
//!
//! ```rust,no_run
//! use media_hub::{MediaHubState, routes};
//! use axum::Router;
//!
//! async fn create_router(state: MediaHubState) -> Router {
//!     Router::new()
//!         .merge(routes::media_routes())
//!         .with_state(state)
//! }
//! ```

pub mod models;
pub mod routes;
pub mod search;
pub mod templates;

use sqlx::SqlitePool;
use std::sync::Arc;

/// Media Hub application state
#[derive(Clone)]
pub struct MediaHubState {
    /// Database connection pool
    pub pool: SqlitePool,

    /// Storage directory path
    pub storage_dir: String,

    /// Access control service
    pub access_control: Arc<access_control::AccessControlService>,
}

impl MediaHubState {
    /// Create a new MediaHubState
    pub fn new(pool: SqlitePool, storage_dir: String) -> Self {
        let access_control = Arc::new(access_control::AccessControlService::with_audit_enabled(
            pool.clone(),
            true,
        ));

        Self {
            pool,
            storage_dir,
            access_control,
        }
    }
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize media hub
pub async fn init() -> anyhow::Result<()> {
    tracing::info!("Media Hub initialized (version {})", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_init() {
        let result = init().await;
        assert!(result.is_ok());
    }
}

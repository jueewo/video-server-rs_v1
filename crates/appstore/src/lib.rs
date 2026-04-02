//! Appstore template registry.
//!
//! App templates are stored as directories under `storage/appstore/`.
//! Each template directory contains:
//! - `manifest.yaml`  – template metadata (name, runtime, entry point, etc.)
//! - `schema.json`    – JSON schema for validating folder content data
//! - App code files   – HTML, JS, CSS, etc.
//!
//! Users install a template into a workspace folder by creating an `app.yaml`
//! that references the template id. On publish, the template code and folder
//! data are merged into a self-contained snapshot.

mod app_yaml;
pub mod data_format;
mod preview;
mod registry;
mod routes;

pub use app_yaml::AppConfig;
pub use registry::{AppTemplate, AppTemplateRegistry, RuntimeType};
pub use routes::appstore_routes;

use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

/// Shared state for appstore routes.
#[derive(Clone)]
pub struct AppstoreState {
    pub registry: Arc<AppTemplateRegistry>,
    pub pool: SqlitePool,
    pub storage_base: PathBuf,
}

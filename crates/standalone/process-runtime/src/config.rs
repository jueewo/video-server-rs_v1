//! Environment-based configuration for the process runtime.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP port to listen on.
    pub port: u16,
    /// SQLite database URL (own DB for instances/tasks/schedules).
    pub database_url: String,
    /// Storage directory for agent memory and working files.
    pub storage_dir: PathBuf,
    /// Directory to scan for .yaml/.bpmn process definition files.
    pub sync_dir: Option<PathBuf>,
    /// Main server URL for HTTP-based definition sync.
    pub main_server_url: Option<String>,
    /// Access code for reading definitions from main server.
    pub access_code: Option<String>,
    /// Path to main server's media.db (shared volume, for LLM provider fallback).
    pub main_db_path: Option<PathBuf>,
    /// Default user ID for all operations (sidecar trust model).
    pub default_user_id: String,
    /// Sync interval in seconds.
    pub sync_interval_secs: u64,
    /// Optional API token for bearer auth (if exposed to untrusted network).
    pub api_token: Option<String>,
}

impl Config {
    /// Load configuration from environment variables and CLI args.
    pub fn from_env() -> anyhow::Result<Self> {
        let port = parse_port_from_args()
            .or_else(|| std::env::var("PORT").ok()?.parse().ok())
            .unwrap_or(4100);

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:process.db?mode=rwc".to_string());

        let storage_dir = std::env::var("STORAGE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));

        let sync_dir = std::env::var("SYNC_DIR").ok().map(PathBuf::from);

        let main_server_url = std::env::var("MAIN_SERVER_URL").ok();
        let access_code = std::env::var("ACCESS_CODE").ok();
        let main_db_path = std::env::var("MAIN_DB_PATH").ok().map(PathBuf::from);

        let default_user_id = std::env::var("DEFAULT_USER_ID")
            .unwrap_or_else(|_| "process-runtime".to_string());

        let sync_interval_secs = std::env::var("SYNC_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let api_token = std::env::var("API_TOKEN").ok();

        Ok(Self {
            port,
            database_url,
            storage_dir,
            sync_dir,
            main_server_url,
            access_code,
            main_db_path,
            default_user_id,
            sync_interval_secs,
            api_token,
        })
    }
}

/// Parse `--port=NNNN` from CLI arguments.
fn parse_port_from_args() -> Option<u16> {
    for arg in std::env::args().skip(1) {
        if let Some(port_str) = arg.strip_prefix("--port=") {
            return port_str.parse().ok();
        }
    }
    None
}

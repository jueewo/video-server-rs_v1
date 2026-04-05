use std::fs;

/// Visual identity / white-label configuration.
/// Loaded from `branding.yaml`.
#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub name: String,
    pub logo: String,
    #[serde(default)]
    pub favicon: Option<String>,
    #[serde(default)]
    pub primary_color: Option<String>,
    #[serde(default)]
    pub support_email: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Appkask".to_string(),
            logo: "/static/icon.webp".to_string(),
            favicon: None,
            primary_color: None,
            support_email: None,
            description: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        match fs::read_to_string("branding.yaml") {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
                println!("Failed to parse branding.yaml: {}", e);
                Self::default()
            }),
            Err(_) => {
                println!("No branding.yaml found, using defaults");
                Self::default()
            }
        }
    }
}

/// Deployment topology configuration.
/// Loaded from `config.yaml`. Affects security posture and data scoping.
#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct DeploymentConfig {
    #[serde(default)]
    pub deployment_mode: DeploymentMode,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: String,
    #[serde(default)]
    pub tenant_name: Option<String>,
    /// Unique server identity for federation. Auto-generated UUID if not set.
    #[serde(default = "generate_server_id")]
    pub server_id: String,
    /// Public URL of this server (required for federation).
    #[serde(default)]
    pub server_url: Option<String>,
    /// Enable federation features (pull-based catalog sharing).
    #[serde(default)]
    pub federation_enabled: bool,
    /// How often to pull catalogs from peers (minutes).
    #[serde(default = "default_sync_interval")]
    pub federation_sync_interval_minutes: u64,
    /// Maximum number of items to cache per peer (0 = unlimited).
    #[serde(default)]
    pub federation_max_items_per_peer: i32,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentMode {
    Hosted,
    Standalone,
}

impl Default for DeploymentMode {
    fn default() -> Self { DeploymentMode::Hosted }
}

fn default_tenant_id() -> String { "platform".to_string() }
fn generate_server_id() -> String { uuid::Uuid::new_v4().to_string() }
fn default_sync_interval() -> u64 { 15 }

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            deployment_mode: DeploymentMode::Hosted,
            tenant_id: "platform".to_string(),
            tenant_name: None,
            server_id: generate_server_id(),
            server_url: None,
            federation_enabled: false,
            federation_sync_interval_minutes: 15,
            federation_max_items_per_peer: 0,
        }
    }
}

impl DeploymentConfig {
    pub fn load() -> Self {
        match fs::read_to_string("config.yaml") {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
                println!("\u{26a0}\u{fe0f}  Failed to parse config.yaml: {}", e);
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn is_standalone(&self) -> bool {
        self.deployment_mode == DeploymentMode::Standalone
    }
}

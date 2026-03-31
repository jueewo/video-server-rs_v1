//! app.yaml — the file a workspace folder uses to reference an appstore template.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Contents of an `app.yaml` file in a workspace folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Appstore template id (e.g. "quiz-app").
    pub template: String,
    /// Display title for this app instance.
    #[serde(default)]
    pub title: String,
    /// Optional description.
    #[serde(default)]
    pub description: String,
}

impl AppConfig {
    /// Read `app.yaml` from a folder. Returns `None` if the file doesn't exist.
    pub fn load(folder: &Path) -> Result<Option<Self>> {
        let path = folder.join("app.yaml");
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        let config: AppConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse {:?}", path))?;
        Ok(Some(config))
    }

    /// Write `app.yaml` to a folder.
    pub fn save(&self, folder: &Path) -> Result<()> {
        let path = folder.join("app.yaml");
        let content = serde_yaml::to_string(self)
            .with_context(|| "Failed to serialize app.yaml")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write {:?}", path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_missing() {
        let dir = TempDir::new().unwrap();
        let config = AppConfig::load(dir.path()).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let config = AppConfig {
            template: "quiz-app".to_string(),
            title: "My Quiz".to_string(),
            description: "A test quiz".to_string(),
        };
        config.save(dir.path()).unwrap();

        let loaded = AppConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.template, "quiz-app");
        assert_eq!(loaded.title, "My Quiz");
    }
}

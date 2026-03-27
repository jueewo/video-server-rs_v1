//! Workspace configuration management (workspace.yaml)
//!
//! The workspace.yaml file serves as a manifest and metadata store for special-purpose folders.
//! It tracks folder types, configurations, and processing instructions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Workspace configuration stored in workspace.yaml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub folders: HashMap<String, FolderConfig>,
}

/// Configuration for a special-purpose folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderConfig {
    #[serde(rename = "type")]
    pub folder_type: FolderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// A transparent newtype around a folder type identifier string.
///
/// Serialises as a plain string so existing `workspace.yaml` files remain valid.
/// The special id `"default"` means "no type assigned" — default folders are not
/// persisted in workspace.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct FolderType(pub String);

impl FolderType {
    pub fn new(id: impl Into<String>) -> Self {
        FolderType(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_default(&self) -> bool {
        self.0 == "default"
    }
}

impl Default for FolderType {
    fn default() -> Self {
        FolderType("default".to_string())
    }
}

impl std::fmt::Display for FolderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl WorkspaceConfig {
    /// Create a new workspace config
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            folders: HashMap::new(),
        }
    }

    /// Load workspace config from workspace.yaml
    pub fn load(workspace_root: &Path) -> Result<Self> {
        let yaml_path = workspace_root.join("workspace.yaml");

        if !yaml_path.exists() {
            // Create default config if file doesn't exist
            let config = Self::new("Workspace".to_string(), String::new());
            config.save(workspace_root)?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&yaml_path)
            .with_context(|| format!("Failed to read workspace.yaml at {:?}", yaml_path))?;

        let config: WorkspaceConfig = serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse workspace.yaml")?;

        Ok(config)
    }

    /// Save workspace config to workspace.yaml
    pub fn save(&self, workspace_root: &Path) -> Result<()> {
        let yaml_path = workspace_root.join("workspace.yaml");
        let content = serde_yaml::to_string(self)
            .with_context(|| "Failed to serialize workspace config")?;

        std::fs::write(&yaml_path, content)
            .with_context(|| format!("Failed to write workspace.yaml at {:?}", yaml_path))?;

        Ok(())
    }

    /// Add or update a folder in the config.
    /// Default-type folders are not registered — calling this with `default` removes any existing entry.
    pub fn upsert_folder(&mut self, path: String, folder_type: FolderType) {
        if folder_type.is_default() {
            self.folders.remove(&path);
            return;
        }
        self.folders
            .entry(path)
            .and_modify(|config| {
                config.folder_type = folder_type.clone();
            })
            .or_insert_with(|| FolderConfig {
                folder_type,
                description: None,
                metadata: HashMap::new(),
            });
    }

    /// Set folder description
    pub fn set_folder_description(&mut self, path: &str, description: Option<String>) {
        if let Some(folder) = self.folders.get_mut(path) {
            folder.description = description;
        }
    }

    /// Rename a folder (updates the exact key in folders map)
    pub fn rename_folder(&mut self, old_path: &str, new_path: String) -> bool {
        if let Some(config) = self.folders.remove(old_path) {
            self.folders.insert(new_path, config);
            true
        } else {
            false
        }
    }

    /// Rename a folder and update all sub-folder entries whose paths start with
    /// `old_path/`. This handles both direct renames and moves of parent folders.
    pub fn rename_folder_prefix(&mut self, old_path: &str, new_path: &str) {
        let prefix = format!("{}/", old_path);
        let keys: Vec<String> = self
            .folders
            .keys()
            .filter(|k| *k == old_path || k.starts_with(&prefix))
            .cloned()
            .collect();

        for old_key in keys {
            if let Some(config) = self.folders.remove(&old_key) {
                let new_key = if old_key == old_path {
                    new_path.to_string()
                } else {
                    format!("{}/{}", new_path, &old_key[old_path.len() + 1..])
                };
                self.folders.insert(new_key, config);
            }
        }
    }

    /// Copy folder config (and any sub-folder configs) from one path to another.
    /// Used when duplicating or copying typed folders.
    pub fn copy_folder_prefix(&mut self, from_prefix: &str, to_prefix: &str) {
        let entries: Vec<(String, FolderConfig)> = self
            .folders
            .iter()
            .filter(|(k, _)| *k == from_prefix || k.starts_with(&format!("{}/", from_prefix)))
            .map(|(k, v)| {
                let new_key = if *k == from_prefix {
                    to_prefix.to_string()
                } else {
                    format!("{}{}", to_prefix, &k[from_prefix.len()..])
                };
                (new_key, v.clone())
            })
            .collect();
        for (k, v) in entries {
            self.folders.insert(k, v);
        }
    }

    /// Remove a folder from the config
    pub fn remove_folder(&mut self, path: &str) -> bool {
        self.folders.remove(path).is_some()
    }

    /// Get folder config
    pub fn get_folder(&self, path: &str) -> Option<&FolderConfig> {
        self.folders.get(path)
    }

    /// Update folder metadata
    pub fn set_folder_metadata(&mut self, path: &str, key: String, value: serde_yaml::Value) {
        if let Some(folder) = self.folders.get_mut(path) {
            folder.metadata.insert(key, value);
        }
    }

    /// Sync config with actual filesystem folder structure
    /// Adds missing folders, removes deleted folders
    pub fn sync_with_filesystem(&mut self, workspace_root: &Path) -> Result<()> {
        let mut found_folders = std::collections::HashSet::new();

        // Walk the workspace directory
        if workspace_root.exists() {
            for entry in walkdir::WalkDir::new(workspace_root)
                .min_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_dir())
            {
                // Get relative path from workspace root
                let rel_path = entry
                    .path()
                    .strip_prefix(workspace_root)
                    .ok()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string());

                if let Some(path) = rel_path {
                    found_folders.insert(path.clone());
                    // Default-type folders are not registered — only track existing registered ones
                }
            }
        }

        // Remove folders from config that don't exist on filesystem
        self.folders.retain(|path, _| found_folders.contains(path));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_new() {
        let config = WorkspaceConfig::new("Test".to_string(), "Description".to_string());
        assert_eq!(config.name, "Test");
        assert_eq!(config.description, "Description");
        assert!(config.folders.is_empty());
    }

    #[test]
    fn test_upsert_folder() {
        let mut config = WorkspaceConfig::new("Test".to_string(), String::new());
        config.upsert_folder("agents".to_string(), FolderType::new("agent-collection"));

        assert_eq!(config.folders.len(), 1);
        assert!(config.folders.contains_key("agents"));
        assert_eq!(
            config.folders.get("agents").unwrap().folder_type.as_str(),
            "agent-collection"
        );
    }

    #[test]
    fn test_upsert_folder_updates_type() {
        let mut config = WorkspaceConfig::new("Test".to_string(), String::new());

        // Default type → folder is removed/not registered
        config.upsert_folder("my-folder".to_string(), FolderType::default());
        assert!(!config.folders.contains_key("my-folder"));

        // Insert as static-site
        config.upsert_folder("my-folder".to_string(), FolderType::new("static-site"));
        assert_eq!(
            config.folders.get("my-folder").unwrap().folder_type.as_str(),
            "static-site"
        );

        // Update to course
        config.upsert_folder("my-folder".to_string(), FolderType::new("course"));
        assert_eq!(
            config.folders.get("my-folder").unwrap().folder_type.as_str(),
            "course"
        );

        // Should still be only one folder
        assert_eq!(config.folders.len(), 1);
    }

    #[test]
    fn test_remove_folder() {
        let mut config = WorkspaceConfig::new("Test".to_string(), String::new());
        config.upsert_folder("temp".to_string(), FolderType::new("documentation"));

        assert!(config.remove_folder("temp"));
        assert!(!config.folders.contains_key("temp"));
        assert!(!config.remove_folder("nonexistent"));
    }

    #[test]
    fn test_folder_metadata() {
        let mut config = WorkspaceConfig::new("Test".to_string(), String::new());
        config.upsert_folder("agents".to_string(), FolderType::new("agent-collection"));

        config.set_folder_metadata(
            "agents",
            "model".to_string(),
            serde_yaml::Value::String("claude-sonnet-4.5".to_string()),
        );

        let folder = config.get_folder("agents").unwrap();
        assert!(folder.metadata.contains_key("model"));
    }

    #[test]
    fn test_folder_type_is_default() {
        assert!(FolderType::default().is_default());
        assert!(FolderType::new("default").is_default());
        assert!(!FolderType::new("course").is_default());
    }

    #[test]
    fn test_folder_type_serde_transparent() {
        // Serialises as a plain string
        let ft = FolderType::new("course");
        let serialised = serde_yaml::to_string(&ft).unwrap();
        assert!(serialised.trim() == "course");

        // Deserialises from a plain string
        let de: FolderType = serde_yaml::from_str("static-site").unwrap();
        assert_eq!(de.as_str(), "static-site");
    }
}

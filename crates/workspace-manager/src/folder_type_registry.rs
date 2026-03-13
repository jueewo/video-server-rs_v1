//! Config-driven folder type registry.
//!
//! Folder types are defined as YAML files on disk under `storage/folder-type-registry/`.
//! Six built-in types are embedded in the binary and written to disk on first startup.
//! Users can freely edit the files after that to customise icons, metadata schemas, etc.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================================
// Built-in type definitions (embedded at compile time)
// ============================================================================

const BUILTIN_COURSE: &str = include_str!("builtin_types/course.yaml");
const BUILTIN_STATIC_SITE: &str = include_str!("builtin_types/static-site.yaml");
const BUILTIN_BPMN_SIMULATOR: &str = include_str!("builtin_types/bpmn-simulator.yaml");
const BUILTIN_AGENT_COLLECTION: &str = include_str!("builtin_types/agent-collection.yaml");
const BUILTIN_DOCUMENTATION: &str = include_str!("builtin_types/documentation.yaml");
const BUILTIN_DATA_PIPELINE: &str = include_str!("builtin_types/data-pipeline.yaml");
const BUILTIN_JS_TOOL: &str = include_str!("builtin_types/js-tool.yaml");
const BUILTIN_MEDIA_SERVER: &str = include_str!("builtin_types/media-server.yaml");
const BUILTIN_PRESENTATION: &str = include_str!("builtin_types/presentation.yaml");
const BUILTIN_YHM_SITE_DATA: &str = include_str!("builtin_types/yhm-site-data.yaml");

const BUILTINS: &[(&str, &str)] = &[
    ("course.yaml", BUILTIN_COURSE),
    ("static-site.yaml", BUILTIN_STATIC_SITE),
    ("bpmn-simulator.yaml", BUILTIN_BPMN_SIMULATOR),
    ("agent-collection.yaml", BUILTIN_AGENT_COLLECTION),
    ("documentation.yaml", BUILTIN_DOCUMENTATION),
    ("data-pipeline.yaml", BUILTIN_DATA_PIPELINE),
    ("js-tool.yaml", BUILTIN_JS_TOOL),
    ("media-server.yaml", BUILTIN_MEDIA_SERVER),
    ("presentation.yaml", BUILTIN_PRESENTATION),
    ("yhm-site-data.yaml", BUILTIN_YHM_SITE_DATA),
];

// ============================================================================
// Data types
// ============================================================================

/// The type of a metadata field value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Enum,
    Multiline,
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::String
    }
}

/// A single field in a folder type's metadata schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type", default)]
    pub field_type: FieldType,
    /// Allowed values for `enum` fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    /// Default value serialised as a YAML value.
    #[serde(default = "default_yaml_null")]
    pub default: serde_yaml::Value,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_yaml_null() -> serde_yaml::Value {
    serde_yaml::Value::Null
}

/// A link from a folder type to an app that can open folders of that type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLink {
    pub app_id: String,
    pub label: String,
    pub icon: String,
    /// URL template; placeholders: `{workspace_id}`, `{folder_path}`, `{slug}`.
    pub url_template: String,
}

/// A folder type definition loaded from a YAML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTypeDefinition {
    pub id: String,
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Optional git repository URL to clone when initialising a folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_template: Option<String>,
    #[serde(default)]
    pub metadata_schema: Vec<MetadataField>,
    /// Apps that can open folders of this type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apps: Vec<AppLink>,
}

// ============================================================================
// Registry
// ============================================================================

/// In-memory cache of all folder type definitions, backed by YAML files on disk.
#[derive(Debug, Clone)]
pub struct FolderTypeRegistry {
    /// Directory where YAML files are stored.
    registry_dir: PathBuf,
    /// Cached definitions keyed by type id.
    types: HashMap<String, FolderTypeDefinition>,
}

impl FolderTypeRegistry {
    /// Write the six built-in YAML files to `registry_dir` if they do not already exist.
    /// Creates the directory if necessary.
    pub fn ensure_defaults(registry_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(registry_dir)
            .with_context(|| format!("Failed to create registry dir {:?}", registry_dir))?;

        for (filename, content) in BUILTINS {
            let path = registry_dir.join(filename);
            if !path.exists() {
                std::fs::write(&path, content).with_context(|| {
                    format!("Failed to write builtin type file {:?}", path)
                })?;
            }
        }

        Ok(())
    }

    /// Load all `*.yaml` files from `registry_dir` into memory.
    pub fn load(registry_dir: &Path) -> Result<Self> {
        let mut types = HashMap::new();

        if registry_dir.exists() {
            for entry in std::fs::read_dir(registry_dir)
                .with_context(|| format!("Failed to read registry dir {:?}", registry_dir))?
            {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                    continue;
                }

                match Self::load_file(&path) {
                    Ok(def) => {
                        types.insert(def.id.clone(), def);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load folder type from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(Self {
            registry_dir: registry_dir.to_path_buf(),
            types,
        })
    }

    fn load_file(path: &Path) -> Result<FolderTypeDefinition> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        let def: FolderTypeDefinition = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML in {:?}", path))?;
        Ok(def)
    }

    /// Return all registered type definitions sorted by id.
    pub fn list_types(&self) -> Vec<&FolderTypeDefinition> {
        let mut list: Vec<&FolderTypeDefinition> = self.types.values().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Look up a type by id.
    pub fn get_type(&self, id: &str) -> Option<&FolderTypeDefinition> {
        self.types.get(id)
    }

    /// Save a new type definition to disk and add it to the cache.
    /// Returns an error if a type with the same id already exists.
    pub fn create_type(&mut self, def: FolderTypeDefinition) -> Result<()> {
        if self.types.contains_key(&def.id) {
            anyhow::bail!("Folder type '{}' already exists", def.id);
        }
        self.save_to_disk(&def)?;
        self.types.insert(def.id.clone(), def);
        Ok(())
    }

    /// Replace an existing type definition on disk and in the cache.
    pub fn update_type(&mut self, id: &str, mut def: FolderTypeDefinition) -> Result<()> {
        // Remove old file if the id changed (shouldn't happen via API but be safe)
        if id != def.id {
            let old_path = self.type_path(id);
            if old_path.exists() {
                std::fs::remove_file(&old_path)
                    .with_context(|| format!("Failed to remove old type file {:?}", old_path))?;
            }
            self.types.remove(id);
        }
        // Ensure the id in the struct matches
        def.id = def.id.clone();
        self.save_to_disk(&def)?;
        self.types.insert(def.id.clone(), def);
        Ok(())
    }

    /// Remove a type from disk and the cache.
    pub fn delete_type(&mut self, id: &str) -> Result<()> {
        let path = self.type_path(id);
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to delete type file {:?}", path))?;
        }
        self.types.remove(id);
        Ok(())
    }

    /// For each field in the type's schema that is absent from `metadata`, insert the
    /// field's default value.  Existing values are never overwritten.
    pub fn apply_defaults(
        &self,
        type_id: &str,
        metadata: &mut HashMap<String, serde_yaml::Value>,
    ) {
        if let Some(def) = self.types.get(type_id) {
            for field in &def.metadata_schema {
                if !metadata.contains_key(&field.key) && field.default != serde_yaml::Value::Null {
                    metadata.insert(field.key.clone(), field.default.clone());
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn type_path(&self, id: &str) -> PathBuf {
        self.registry_dir.join(format!("{}.yaml", id))
    }

    fn save_to_disk(&self, def: &FolderTypeDefinition) -> Result<()> {
        let path = self.type_path(&def.id);
        let content = serde_yaml::to_string(def)
            .with_context(|| format!("Failed to serialise type '{}'", def.id))?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write type file {:?}", path))?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_registry() -> (TempDir, FolderTypeRegistry) {
        let dir = TempDir::new().unwrap();
        FolderTypeRegistry::ensure_defaults(dir.path()).unwrap();
        let registry = FolderTypeRegistry::load(dir.path()).unwrap();
        (dir, registry)
    }

    #[test]
    fn test_ensure_defaults_creates_files() {
        let dir = TempDir::new().unwrap();
        FolderTypeRegistry::ensure_defaults(dir.path()).unwrap();

        for (filename, _) in BUILTINS {
            assert!(dir.path().join(filename).exists(), "Missing {}", filename);
        }
    }

    #[test]
    fn test_load_builtin_types() {
        let (_dir, registry) = make_registry();
        assert_eq!(registry.list_types().len(), 7);
        assert!(registry.get_type("course").is_some());
        assert!(registry.get_type("static-site").is_some());
        assert!(registry.get_type("bpmn-simulator").is_some());
        assert!(registry.get_type("agent-collection").is_some());
        assert!(registry.get_type("documentation").is_some());
        assert!(registry.get_type("data-pipeline").is_some());
        assert!(registry.get_type("js-tool").is_some());
    }

    #[test]
    fn test_js_tool_has_app_link() {
        let (_dir, registry) = make_registry();
        let js_tool = registry.get_type("js-tool").unwrap();
        assert_eq!(js_tool.apps.len(), 1);
        assert_eq!(js_tool.apps[0].app_id, "js-tool-viewer");
        assert!(js_tool.apps[0].url_template.contains("{workspace_id}"));
    }

    #[test]
    fn test_create_and_get_type() {
        let (dir, mut registry) = make_registry();
        let def = FolderTypeDefinition {
            id: "my-type".to_string(),
            name: "My Type".to_string(),
            icon: "star".to_string(),
            description: "A test type".to_string(),
            color: None,
            git_template: None,
            metadata_schema: vec![],
            apps: vec![],
        };
        registry.create_type(def.clone()).unwrap();

        // File exists on disk
        assert!(dir.path().join("my-type.yaml").exists());

        // Retrievable from cache
        let got = registry.get_type("my-type").unwrap();
        assert_eq!(got.name, "My Type");
    }

    #[test]
    fn test_create_type_duplicate_fails() {
        let (_dir, mut registry) = make_registry();
        let def = FolderTypeDefinition {
            id: "course".to_string(),
            name: "Duplicate".to_string(),
            icon: "x".to_string(),
            description: String::new(),
            color: None,
            git_template: None,
            metadata_schema: vec![],
            apps: vec![],
        };
        assert!(registry.create_type(def).is_err());
    }

    #[test]
    fn test_delete_type() {
        let (dir, mut registry) = make_registry();
        registry.delete_type("course").unwrap();
        assert!(registry.get_type("course").is_none());
        assert!(!dir.path().join("course.yaml").exists());
    }

    #[test]
    fn test_apply_defaults() {
        let (_dir, registry) = make_registry();
        let mut meta: HashMap<String, serde_yaml::Value> = HashMap::new();
        // pre-set "level" — should NOT be overwritten
        meta.insert(
            "level".to_string(),
            serde_yaml::Value::String("advanced".to_string()),
        );

        registry.apply_defaults("course", &mut meta);

        // title default is "" (non-null) → inserted
        assert!(meta.contains_key("title"));
        // level was pre-set → must not have changed
        assert_eq!(
            meta.get("level").unwrap(),
            &serde_yaml::Value::String("advanced".to_string())
        );
        // instructor default is "" → inserted
        assert!(meta.contains_key("instructor"));
    }

    #[test]
    fn test_ensure_defaults_does_not_overwrite() {
        let dir = TempDir::new().unwrap();
        FolderTypeRegistry::ensure_defaults(dir.path()).unwrap();

        // Overwrite course.yaml with garbage
        let course_path = dir.path().join("course.yaml");
        std::fs::write(&course_path, "id: custom\nname: Custom\nicon: x\n").unwrap();

        // ensure_defaults must NOT overwrite it
        FolderTypeRegistry::ensure_defaults(dir.path()).unwrap();
        let content = std::fs::read_to_string(&course_path).unwrap();
        assert!(content.contains("id: custom"));
    }
}

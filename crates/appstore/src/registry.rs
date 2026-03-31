//! Template registry backed by YAML manifest files on disk.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How the app is served at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    /// Static HTML/JS/CSS served directly.
    Static,
    /// Bun sidecar process (has server.ts/server.js).
    Bun,
    /// Custom server binary (meta.yaml server_command).
    Custom,
}

impl Default for RuntimeType {
    fn default() -> Self {
        RuntimeType::Static
    }
}

/// A template manifest loaded from `manifest.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTemplate {
    /// Unique template identifier (matches directory name).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Category for grouping in the UI.
    #[serde(default)]
    pub category: String,
    /// Version string.
    #[serde(default = "default_version")]
    pub version: String,
    /// Icon identifier for the UI.
    #[serde(default = "default_icon")]
    pub icon: String,
    /// Tailwind color for the UI.
    #[serde(default = "default_color")]
    pub color: String,
    /// Runtime type: how to serve the app.
    #[serde(default)]
    pub runtime: RuntimeType,
    /// Entry point file (e.g. "index.html" or "server.ts").
    #[serde(default = "default_entry")]
    pub entry: String,
    /// Path to JSON schema file (relative to template dir).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Data files expected in the workspace folder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_files: Vec<DataFileSpec>,
}

/// Describes a data file the template expects in the workspace folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFileSpec {
    /// Filename (e.g. "questions.json").
    pub file: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Whether this file is required.
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_version() -> String {
    "1.0.0".to_string()
}
fn default_icon() -> String {
    "puzzle".to_string()
}
fn default_color() -> String {
    "primary".to_string()
}
fn default_entry() -> String {
    "index.html".to_string()
}
fn default_true() -> bool {
    true
}

/// In-memory cache of all app templates, backed by directories on disk.
#[derive(Debug, Clone)]
pub struct AppTemplateRegistry {
    /// Root directory where template directories live.
    registry_dir: PathBuf,
    /// Cached templates keyed by id.
    templates: HashMap<String, AppTemplate>,
}

impl AppTemplateRegistry {
    /// Load all templates from `registry_dir`.
    ///
    /// Each subdirectory containing a `manifest.yaml` is treated as a template.
    pub fn load(registry_dir: &Path) -> Result<Self> {
        let mut templates = HashMap::new();

        if !registry_dir.exists() {
            std::fs::create_dir_all(registry_dir)
                .with_context(|| format!("Failed to create appstore dir {:?}", registry_dir))?;
        }

        for entry in std::fs::read_dir(registry_dir)
            .with_context(|| format!("Failed to read appstore dir {:?}", registry_dir))?
        {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.yaml");
            if !manifest_path.exists() {
                continue;
            }

            match Self::load_manifest(&manifest_path) {
                Ok(template) => {
                    templates.insert(template.id.clone(), template);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load appstore template from {:?}: {}",
                        manifest_path,
                        e
                    );
                }
            }
        }

        Ok(Self {
            registry_dir: registry_dir.to_path_buf(),
            templates,
        })
    }

    fn load_manifest(path: &Path) -> Result<AppTemplate> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        let template: AppTemplate = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest {:?}", path))?;
        Ok(template)
    }

    /// Return all registered templates sorted by name.
    pub fn list(&self) -> Vec<&AppTemplate> {
        let mut list: Vec<&AppTemplate> = self.templates.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Look up a template by id.
    pub fn get(&self, id: &str) -> Option<&AppTemplate> {
        self.templates.get(id)
    }

    /// Return the directory path for a template's files.
    pub fn template_dir(&self, id: &str) -> PathBuf {
        self.registry_dir.join(id)
    }

    /// Return the root directory of the appstore.
    pub fn registry_dir(&self) -> &Path {
        &self.registry_dir
    }

    /// Load the JSON schema for a template, if it has one.
    pub fn load_schema(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let template = match self.get(id) {
            Some(t) => t,
            None => anyhow::bail!("Template '{}' not found", id),
        };

        let schema_file = match &template.schema {
            Some(s) => s,
            None => return Ok(None),
        };

        let schema_path = self.template_dir(id).join(schema_file);
        if !schema_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema {:?}", schema_path))?;
        let schema: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse schema {:?}", schema_path))?;
        Ok(Some(schema))
    }

    /// Merge template code + folder data into a self-contained snapshot directory.
    ///
    /// 1. Copies all template files (except manifest.yaml, schema.json, sample-*) to `dst/`
    /// 2. Copies all folder data files to `dst/data/`
    /// 3. Skips `app.yaml` from the folder (it's metadata, not served content)
    ///
    /// The `dst` directory is created if it doesn't exist.
    pub fn merge_to_snapshot(
        &self,
        template_id: &str,
        folder_src: &Path,
        dst: &Path,
    ) -> Result<()> {
        let template = self
            .get(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        let tmpl_dir = self.template_dir(template_id);
        if !tmpl_dir.exists() {
            anyhow::bail!("Template directory {:?} does not exist", tmpl_dir);
        }

        // Create destination
        std::fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create snapshot dir {:?}", dst))?;

        // 1. Copy template files (skip manifest, schema, sample files)
        let skip_files: &[&str] = &["manifest.yaml", "schema.json"];
        copy_dir_filtered(&tmpl_dir, dst, &|name: &str| {
            !skip_files.contains(&name) && !name.starts_with("sample-")
        })?;

        // 2. Copy folder data files to dst/data/
        let data_dst = dst.join("data");
        std::fs::create_dir_all(&data_dst)
            .with_context(|| format!("Failed to create data dir {:?}", data_dst))?;

        // Copy expected data files
        for spec in &template.data_files {
            let src_file = folder_src.join(&spec.file);
            if src_file.exists() {
                let dst_file = data_dst.join(&spec.file);
                if let Some(parent) = dst_file.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src_file, &dst_file).with_context(|| {
                    format!("Failed to copy data file {:?}", spec.file)
                })?;
            }
        }

        // Also copy any other files from the folder that aren't app.yaml or
        // already-copied data files (e.g. assets/, custom CSS, images)
        let data_file_names: Vec<&str> = template.data_files.iter().map(|d| d.file.as_str()).collect();
        if folder_src.exists() {
            for entry in std::fs::read_dir(folder_src)? {
                let entry = entry?;
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Skip app.yaml, already-copied data files, and hidden files
                if name_str == "app.yaml"
                    || data_file_names.contains(&name_str.as_ref())
                    || name_str.starts_with('.')
                {
                    continue;
                }

                let src_path = entry.path();
                let dst_path = data_dst.join(&*name_str);

                if src_path.is_dir() {
                    copy_dir_filtered(&src_path, &dst_path, &|_| true)?;
                } else {
                    std::fs::copy(&src_path, &dst_path)?;
                }
            }
        }

        Ok(())
    }
}

/// Recursively copy a directory, applying a filename filter.
fn copy_dir_filtered(
    src: &Path,
    dst: &Path,
    filter: &dyn Fn(&str) -> bool,
) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if !filter(&name_str) {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&*name_str);

        if src_path.is_dir() {
            copy_dir_filtered(&src_path, &dst_path, &|_| true)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template(dir: &Path, id: &str) {
        let template_dir = dir.join(id);
        fs::create_dir_all(&template_dir).unwrap();
        let manifest = format!(
            r#"id: {id}
name: Test {id}
description: A test template
category: testing
runtime: static
entry: index.html
schema: schema.json
data_files:
  - file: data.json
    description: Test data
    required: true
"#
        );
        fs::write(template_dir.join("manifest.yaml"), manifest).unwrap();
        fs::write(
            template_dir.join("schema.json"),
            r#"{"type": "object"}"#,
        )
        .unwrap();
        fs::write(
            template_dir.join("index.html"),
            "<html><body>Test</body></html>",
        )
        .unwrap();
    }

    #[test]
    fn test_load_empty_dir() {
        let dir = TempDir::new().unwrap();
        let registry = AppTemplateRegistry::load(dir.path()).unwrap();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_load_template() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path(), "quiz-app");

        let registry = AppTemplateRegistry::load(dir.path()).unwrap();
        assert_eq!(registry.list().len(), 1);

        let t = registry.get("quiz-app").unwrap();
        assert_eq!(t.name, "Test quiz-app");
        assert_eq!(t.runtime, RuntimeType::Static);
        assert_eq!(t.data_files.len(), 1);
    }

    #[test]
    fn test_load_schema() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path(), "quiz-app");

        let registry = AppTemplateRegistry::load(dir.path()).unwrap();
        let schema = registry.load_schema("quiz-app").unwrap();
        assert!(schema.is_some());
    }

    #[test]
    fn test_template_dir() {
        let dir = TempDir::new().unwrap();
        let registry = AppTemplateRegistry::load(dir.path()).unwrap();
        assert_eq!(registry.template_dir("quiz-app"), dir.path().join("quiz-app"));
    }

    #[test]
    fn test_skips_dirs_without_manifest() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("no-manifest")).unwrap();
        fs::write(dir.path().join("no-manifest/readme.txt"), "hi").unwrap();

        let registry = AppTemplateRegistry::load(dir.path()).unwrap();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_creates_dir_if_missing() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("appstore");
        assert!(!sub.exists());

        let registry = AppTemplateRegistry::load(&sub).unwrap();
        assert!(sub.exists());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_merge_to_snapshot() {
        let dir = TempDir::new().unwrap();
        create_test_template(dir.path(), "quiz-app");
        // Add a sample file that should be skipped
        fs::write(dir.path().join("quiz-app/sample-questions.json"), "{}").unwrap();

        let registry = AppTemplateRegistry::load(dir.path()).unwrap();

        // Create a mock workspace folder with data
        let folder = TempDir::new().unwrap();
        fs::write(
            folder.path().join("app.yaml"),
            "template: quiz-app\ntitle: My Quiz",
        )
        .unwrap();
        fs::write(folder.path().join("data.json"), r#"{"questions":[]}"#).unwrap();
        fs::create_dir_all(folder.path().join("assets")).unwrap();
        fs::write(folder.path().join("assets/logo.png"), "PNG").unwrap();

        // Merge
        let snapshot = TempDir::new().unwrap();
        registry
            .merge_to_snapshot("quiz-app", folder.path(), snapshot.path())
            .unwrap();

        // Template code is in root
        assert!(snapshot.path().join("index.html").exists());
        // manifest.yaml and schema.json are NOT copied
        assert!(!snapshot.path().join("manifest.yaml").exists());
        assert!(!snapshot.path().join("schema.json").exists());
        // sample-* files are NOT copied
        assert!(!snapshot.path().join("sample-questions.json").exists());
        // Data files are in data/
        assert!(snapshot.path().join("data/data.json").exists());
        // Extra folder content (assets) is in data/
        assert!(snapshot.path().join("data/assets/logo.png").exists());
        // app.yaml is NOT in the snapshot
        assert!(!snapshot.path().join("data/app.yaml").exists());
    }
}

//! Course Processor
//!
//! Manages structured online courses with modules and lessons.
//! Courses are authored in workspace folders and published to standalone viewers.

use anyhow::{Context, Result};
use std::path::Path;

/// Course configuration (course.yaml in course folder)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CourseConfig {
    /// Course title
    pub title: String,

    /// Course description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Instructor name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructor: Option<String>,

    /// Course level (beginner, intermediate, advanced)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,

    /// Course modules
    #[serde(default)]
    pub modules: Vec<Module>,

    /// Entry point file (e.g., "index.md")
    #[serde(default = "default_entry_point")]
    pub entry_point: String,
}

fn default_entry_point() -> String {
    "index.md".to_string()
}

/// Course module containing lessons
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Module {
    /// Module title
    pub title: String,

    /// Display order
    pub order: i32,

    /// Module description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Lessons in this module
    #[serde(default)]
    pub lessons: Vec<Lesson>,
}

/// Individual lesson within a module
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Lesson {
    /// Lesson title
    pub title: String,

    /// Markdown file path (relative to course folder)
    pub file: String,

    /// Markdown content (populated when generating manifest for publishing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Estimated duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<i32>,

    /// References to media items (access-groups in vaults)
    #[serde(default)]
    pub media_refs: Vec<MediaRef>,
}

/// Reference to a media item stored as access-group in vault
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MediaRef {
    /// Media item slug
    pub slug: String,

    /// Vault ID (optional, can be inferred from context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,

    /// Media type (video, pdf, image, data, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    /// Description of how this media is used in the lesson
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CourseConfig {
    /// Load course configuration from course.yaml
    pub fn load(course_folder: &Path) -> Result<Self> {
        let config_path = course_folder.join("course.yaml");
        let yaml_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read course.yaml at {:?}", config_path))?;

        let config: CourseConfig = serde_yaml::from_str(&yaml_content)
            .with_context(|| format!("Failed to parse course.yaml at {:?}", config_path))?;

        Ok(config)
    }

    /// Save course configuration to course.yaml
    pub fn save(&self, course_folder: &Path) -> Result<()> {
        let config_path = course_folder.join("course.yaml");
        let yaml_content = serde_yaml::to_string(self)
            .context("Failed to serialize course config")?;

        std::fs::write(&config_path, yaml_content)
            .with_context(|| format!("Failed to write course.yaml at {:?}", config_path))?;

        Ok(())
    }

    /// Calculate total duration across all lessons
    pub fn total_duration_minutes(&self) -> i32 {
        self.modules.iter()
            .flat_map(|m| &m.lessons)
            .filter_map(|l| l.duration_minutes)
            .sum()
    }

    /// Count total number of lessons
    pub fn lesson_count(&self) -> usize {
        self.modules.iter()
            .map(|m| m.lessons.len())
            .sum()
    }
}

/// Parsed course structure with validated files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CourseStructure {
    pub title: String,
    pub description: Option<String>,
    pub instructor: Option<String>,
    pub level: Option<String>,
    pub modules: Vec<Module>,
    pub total_duration_minutes: i32,
    pub lesson_count: usize,
}

/// Load and validate course structure
pub fn load_course(folder_path: &Path) -> Result<CourseStructure> {
    tracing::info!("Loading course from {:?}", folder_path);

    let config = CourseConfig::load(folder_path)?;

    // Validate all lesson files exist
    for module in &config.modules {
        for lesson in &module.lessons {
            let lesson_path = folder_path.join(&lesson.file);
            if !lesson_path.exists() {
                anyhow::bail!(
                    "Lesson file not found: {} (module: {})",
                    lesson.file,
                    module.title
                );
            }
        }
    }

    // Validate entry point exists
    let entry_point_path = folder_path.join(&config.entry_point);
    if !entry_point_path.exists() {
        anyhow::bail!("Entry point file not found: {}", config.entry_point);
    }

    // Compute counts before moving config
    let total_duration = config.total_duration_minutes();
    let lesson_cnt = config.lesson_count();

    Ok(CourseStructure {
        title: config.title,
        description: config.description,
        instructor: config.instructor,
        level: config.level,
        total_duration_minutes: total_duration,
        lesson_count: lesson_cnt,
        modules: config.modules,
    })
}

/// Generate course manifest for publishing
/// Returns JSON manifest with course structure + media references + lesson content
pub fn generate_manifest(folder_path: &Path) -> Result<serde_json::Value> {
    tracing::info!("Generating course manifest from {:?}", folder_path);

    let mut course = load_course(folder_path)?;

    // Load lesson content from markdown files
    for module in &mut course.modules {
        for lesson in &mut module.lessons {
            let lesson_path = folder_path.join(&lesson.file);
            let content = std::fs::read_to_string(&lesson_path)
                .with_context(|| format!("Failed to read lesson file: {}", lesson.file))?;
            lesson.content = Some(content);
        }
    }

    // TODO: Validate media references exist in vault

    let manifest = serde_json::to_value(&course)
        .context("Failed to serialize course structure to JSON")?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_config_parsing() {
        let yaml = r#"
title: "Rust Programming"
instructor: "Jane Doe"
level: "beginner"
entry_point: "index.md"
modules:
  - title: "Module 1: Basics"
    order: 1
    description: "Learn Rust fundamentals"
    lessons:
      - title: "Variables"
        file: "01-variables.md"
        duration_minutes: 30
        media_refs:
          - slug: "rust-variables-video"
            vault_id: "vault-123"
            media_type: "video"
            description: "Video tutorial on variables"
"#;
        let config: CourseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.title, "Rust Programming");
        assert_eq!(config.modules.len(), 1);
        assert_eq!(config.modules[0].lessons.len(), 1);
        assert_eq!(config.modules[0].lessons[0].media_refs.len(), 1);
        assert_eq!(config.total_duration_minutes(), 30);
        assert_eq!(config.lesson_count(), 1);
    }

    #[test]
    fn test_default_entry_point() {
        let yaml = r#"
title: "Test Course"
modules: []
"#;
        let config: CourseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.entry_point, "index.md");
    }
}

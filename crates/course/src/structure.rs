use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Optional course.yaml at the root of the course folder.
#[derive(Debug, Deserialize, Default)]
pub struct CourseConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub instructor: Option<String>,
    /// Override module titles and ordering. Key = folder name (e.g. "session1").
    #[serde(default)]
    pub modules: Vec<ModuleConfig>,
    /// Override individual lesson titles. Key = relative path.
    #[serde(default)]
    pub lessons: HashMap<String, LessonConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    pub path: String,
    pub title: Option<String>,
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct LessonConfig {
    pub title: Option<String>,
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseStructure {
    pub title: String,
    pub description: Option<String>,
    pub instructor: Option<String>,
    pub modules: Vec<CourseModule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourseModule {
    /// Relative path from course folder root (e.g. "session1")
    pub path: String,
    pub title: String,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Lesson {
    /// Relative path from course folder root (e.g. "session1/chapter1/intro.md")
    pub path: String,
    pub title: String,
}

/// Load course structure from a folder on disk.
/// Reads optional course.yaml for metadata/ordering overrides.
pub fn load_course(folder_abs: &Path, _folder_path: &str) -> anyhow::Result<CourseStructure> {
    // Load optional course.yaml
    let config: CourseConfig = {
        let yaml_path = folder_abs.join("course.yaml");
        if yaml_path.exists() {
            let content = std::fs::read_to_string(&yaml_path)?;
            serde_yaml::from_str(&content).unwrap_or_default()
        } else {
            CourseConfig::default()
        }
    };

    // Infer title from folder name if not in config
    let title = config.title.unwrap_or_else(|| {
        folder_abs
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| title_case(s))
            .unwrap_or_else(|| "Course".to_string())
    });

    // Collect all markdown files recursively, grouped by top-level subfolder
    let mut module_map: HashMap<String, Vec<(String, String)>> = HashMap::new(); // module_path → [(lesson_path, lesson_name)]

    for entry in WalkDir::new(folder_abs)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" && ext != "mdx" {
            continue;
        }
        let rel = path
            .strip_prefix(folder_abs)?
            .to_string_lossy()
            .replace('\\', "/");

        // Top-level folder = module
        let parts: Vec<&str> = rel.splitn(2, '/').collect();
        let module_key = if parts.len() > 1 {
            parts[0].to_string()
        } else {
            // md file at root level — use empty string as "root module"
            String::new()
        };

        let lesson_name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|s| title_case(s))
            .unwrap_or_else(|| rel.clone());

        module_map
            .entry(module_key)
            .or_default()
            .push((rel, lesson_name));
    }

    // Build module config lookup
    let module_config_map: HashMap<&str, &ModuleConfig> = config
        .modules
        .iter()
        .map(|m| (m.path.as_str(), m))
        .collect();

    // Build modules, applying config overrides
    let mut modules: Vec<(i32, CourseModule)> = module_map
        .into_iter()
        .map(|(module_path, mut lesson_entries)| {
            let mc = module_config_map.get(module_path.as_str());

            let title = mc
                .and_then(|m| m.title.as_deref())
                .unwrap_or(&module_path)
                .to_string();
            let title = if title.is_empty() {
                "Introduction".to_string()
            } else {
                title_case(&title)
            };

            let order = mc.and_then(|m| m.order).unwrap_or(999);

            // Sort lessons: by config order if specified, otherwise alphabetically
            lesson_entries.sort_by(|(path_a, _), (path_b, _)| {
                let order_a = config
                    .lessons
                    .get(path_a)
                    .and_then(|l| l.order)
                    .unwrap_or(999);
                let order_b = config
                    .lessons
                    .get(path_b)
                    .and_then(|l| l.order)
                    .unwrap_or(999);
                if order_a != order_b {
                    order_a.cmp(&order_b)
                } else {
                    path_a.cmp(path_b)
                }
            });

            let lessons = lesson_entries
                .into_iter()
                .map(|(path, default_title)| {
                    let title = config
                        .lessons
                        .get(&path)
                        .and_then(|l| l.title.as_deref())
                        .map(|s| s.to_string())
                        .unwrap_or(default_title);
                    Lesson { path, title }
                })
                .collect();

            (order, CourseModule { path: module_path, title, lessons })
        })
        .collect();

    // Sort modules by order then alphabetically
    modules.sort_by(|(order_a, m_a), (order_b, m_b)| {
        if order_a != order_b {
            order_a.cmp(order_b)
        } else {
            m_a.path.cmp(&m_b.path)
        }
    });

    let modules = modules.into_iter().map(|(_, m)| m).collect();

    Ok(CourseStructure {
        title,
        description: config.description,
        instructor: config.instructor,
        modules,
    })
}

/// Convert a snake_case or kebab-case string to Title Case.
fn title_case(s: &str) -> String {
    s.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

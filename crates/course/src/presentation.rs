use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PresentationConfig {
    pub title: Option<String>,
    pub theme: Option<String>,
    pub transition: Option<String>,
    pub show_progress: Option<bool>,
    pub show_slide_number: Option<String>,
    #[serde(rename = "loop", default)]
    pub loop_: bool,
    pub auto_slide: Option<u32>,
}

pub struct PresentationData {
    pub title: String,
    pub theme: String,
    pub transition: String,
    pub show_progress: bool,
    pub show_slide_number: String,
    pub loop_: bool,
    pub auto_slide: u32,
    pub raw_slides: String,
    pub slide_count: usize,
}

pub fn load_presentation(folder_abs: &Path) -> anyhow::Result<PresentationData> {
    // Load config (optional)
    let config: PresentationConfig = std::fs::read_to_string(folder_abs.join("presentation.yaml"))
        .ok()
        .and_then(|s| serde_yaml::from_str(&s).ok())
        .unwrap_or_default();

    // Load slides content
    let raw_slides = if folder_abs.join("slides.md").exists() {
        std::fs::read_to_string(folder_abs.join("slides.md"))?
    } else {
        discover_slides(folder_abs)
    };

    let slide_count = raw_slides.split("\n---\n").count();

    // Derive title: config → folder name
    let title = config.title.clone().unwrap_or_else(|| {
        folder_abs
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Presentation")
            .replace(['-', '_'], " ")
    });

    Ok(PresentationData {
        title,
        theme: config.theme.clone().unwrap_or_else(|| "white".to_string()),
        transition: config.transition.clone().unwrap_or_else(|| "slide".to_string()),
        show_progress: config.show_progress.unwrap_or(true),
        show_slide_number: config.show_slide_number.clone().unwrap_or_else(|| "all".to_string()),
        loop_: config.loop_,
        auto_slide: config.auto_slide.unwrap_or(0),
        raw_slides,
        slide_count,
    })
}

/// Walk all `.md` files alphabetically; top-level subfolders become section title slides.
pub fn discover_slides(folder_abs: &Path) -> String {
    let mut sections: std::collections::BTreeMap<String, Vec<(String, String)>> =
        std::collections::BTreeMap::new();

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
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename == "slides.md" {
            continue;
        }

        let rel = match path.strip_prefix(folder_abs) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        let parts: Vec<&str> = rel.splitn(2, '/').collect();
        let section_key = if parts.len() > 1 {
            parts[0].to_string()
        } else {
            String::new()
        };

        let content = std::fs::read_to_string(path).unwrap_or_default();
        sections.entry(section_key).or_default().push((rel, content));
    }

    // Sort files within each section
    for files in sections.values_mut() {
        files.sort_by(|a, b| a.0.cmp(&b.0));
    }

    let mut slides: Vec<String> = Vec::new();

    for (section_key, files) in &sections {
        if !section_key.is_empty() {
            let title = section_key
                .replace(['-', '_'], " ")
                .split_whitespace()
                .enumerate()
                .map(|(i, w)| {
                    if i == 0 {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    } else {
                        w.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            slides.push(format!("# {}", title));
        }
        for (_rel, content) in files {
            slides.push(content.clone());
        }
    }

    slides.join("\n\n---\n\n")
}

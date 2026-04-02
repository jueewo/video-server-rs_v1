//! Data file format conversion utility.
//!
//! Templates always fetch data as JSON (e.g. `data/events.json`), but users
//! can write their data files in JSON, YAML, or TOML. When a `.json` file is
//! requested but doesn't exist, we check for `.yaml`, `.yml`, or `.toml`
//! alternatives and convert on the fly.

use std::path::{Path, PathBuf};

/// Try to read a data file, with automatic format conversion.
///
/// If `path` exists, reads it directly. If `path` ends in `.json` and doesn't
/// exist, checks for `.yaml`, `.yml`, or `.toml` alternatives and converts
/// the content to JSON.
///
/// Returns `Some((content_bytes, mime_type))` if a file was found and read.
pub async fn read_data_file(path: &Path) -> Option<(Vec<u8>, &'static str)> {
    // If the file exists as-is, serve it directly
    if path.is_file() {
        let content = tokio::fs::read(path).await.ok()?;
        return Some((content, mime_for_path(path)));
    }

    // Only attempt conversion for .json requests
    let ext = path.extension()?.to_str()?;
    if ext != "json" {
        return None;
    }

    let stem = path.with_extension("");

    // Try YAML variants
    for yaml_ext in &["yaml", "yml"] {
        let yaml_path = stem.with_extension(yaml_ext);
        if yaml_path.is_file() {
            return convert_to_json(&yaml_path, "yaml").await;
        }
    }

    // Try TOML
    let toml_path = stem.with_extension("toml");
    if toml_path.is_file() {
        return convert_to_json(&toml_path, "toml").await;
    }

    None
}

/// Given a directory and a filename (e.g. "events.json"), find the actual file
/// which may have a different extension (.yaml, .yml, .toml).
///
/// Returns the resolved path if found.
pub fn find_data_file(dir: &Path, filename: &str) -> Option<PathBuf> {
    let path = dir.join(filename);
    if path.is_file() {
        return Some(path);
    }

    // Only try alternatives for .json files
    let p = Path::new(filename);
    let ext = p.extension()?.to_str()?;
    if ext != "json" {
        return None;
    }

    let stem = p.with_extension("");
    for alt_ext in &["yaml", "yml", "toml"] {
        let alt = dir.join(stem.with_extension(alt_ext));
        if alt.is_file() {
            return Some(alt);
        }
    }

    None
}

async fn convert_to_json(path: &Path, format: &str) -> Option<(Vec<u8>, &'static str)> {
    let content = tokio::fs::read_to_string(path).await.ok()?;

    let value: serde_json::Value = match format {
        "yaml" => serde_yaml::from_str(&content).ok()?,
        "toml" => toml::from_str(&content).ok()?,
        _ => return None,
    };

    let json = serde_json::to_string_pretty(&value).ok()?;
    Some((json.into_bytes(), "application/json; charset=utf-8"))
}

fn mime_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => "application/json; charset=utf-8",
        Some("yaml") | Some("yml") => "text/yaml; charset=utf-8",
        Some("toml") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

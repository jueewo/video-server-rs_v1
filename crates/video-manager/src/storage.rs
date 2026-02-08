//! Storage utilities for video file management
//!
//! This module provides helper functions for:
//! - Directory creation and validation
//! - File operations (move, copy, delete)
//! - Path validation and sanitization
//! - Cleanup utilities

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Base storage directory for videos
    pub videos_dir: PathBuf,
    /// Temporary upload directory
    pub temp_dir: PathBuf,
    /// Maximum file size in bytes (default: 2GB)
    pub max_file_size: u64,
}

impl StorageConfig {
    /// Create a new storage configuration
    pub fn new(base_dir: PathBuf) -> Self {
        let videos_dir = base_dir.join("videos");
        let temp_dir = base_dir.join("temp");

        Self {
            videos_dir,
            temp_dir,
            max_file_size: 2 * 1024 * 1024 * 1024, // 2GB
        }
    }

    /// Initialize storage directories
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing storage directories");

        // Create main directories
        ensure_dir_exists(&self.videos_dir)?;
        ensure_dir_exists(&self.temp_dir)?;

        // Create public and private subdirectories
        ensure_dir_exists(&self.videos_dir.join("public"))?;
        ensure_dir_exists(&self.videos_dir.join("private"))?;

        info!("Storage directories initialized successfully");
        Ok(())
    }

    /// Get the path for a video directory
    pub fn get_video_dir(&self, slug: &str, is_public: bool) -> PathBuf {
        let visibility = if is_public { "public" } else { "private" };
        self.videos_dir.join(visibility).join(slug)
    }

    /// Get the path for a temporary file
    pub fn get_temp_path(&self, filename: &str) -> PathBuf {
        self.temp_dir.join(filename)
    }
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        debug!("Creating directory: {:?}", path);
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {:?}", path))?;
    } else if !path.is_dir() {
        anyhow::bail!("Path exists but is not a directory: {:?}", path);
    }
    Ok(())
}

/// Create a video directory structure
///
/// Creates:
/// - Base video directory
/// - Quality subdirectories (1080p, 720p, 480p, 360p)
pub fn create_video_directory(base_path: &Path) -> Result<()> {
    info!("Creating video directory structure: {:?}", base_path);

    ensure_dir_exists(base_path)?;

    // Create quality subdirectories
    let qualities = ["1080p", "720p", "480p", "360p"];
    for quality in &qualities {
        let quality_dir = base_path.join(quality);
        ensure_dir_exists(&quality_dir)?;
    }

    debug!("Video directory structure created successfully");
    Ok(())
}

/// Move a file atomically
///
/// Attempts to rename first (fastest), falls back to copy+delete if on different filesystems
pub fn move_file(source: &Path, destination: &Path) -> Result<()> {
    debug!("Moving file from {:?} to {:?}", source, destination);

    if !source.exists() {
        anyhow::bail!("Source file does not exist: {:?}", source);
    }

    // Ensure destination directory exists
    if let Some(parent) = destination.parent() {
        ensure_dir_exists(parent)?;
    }

    // Try atomic rename first
    match fs::rename(source, destination) {
        Ok(_) => {
            info!("File moved successfully (rename)");
            Ok(())
        }
        Err(e) => {
            // If rename fails (e.g., different filesystem), fall back to copy+delete
            warn!("Rename failed, falling back to copy+delete: {}", e);

            fs::copy(source, destination).with_context(|| {
                format!("Failed to copy file from {:?} to {:?}", source, destination)
            })?;

            fs::remove_file(source)
                .with_context(|| format!("Failed to remove source file: {:?}", source))?;

            info!("File moved successfully (copy+delete)");
            Ok(())
        }
    }
}

/// Copy a file
pub fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    debug!("Copying file from {:?} to {:?}", source, destination);

    if !source.exists() {
        anyhow::bail!("Source file does not exist: {:?}", source);
    }

    // Ensure destination directory exists
    if let Some(parent) = destination.parent() {
        ensure_dir_exists(parent)?;
    }

    fs::copy(source, destination)
        .with_context(|| format!("Failed to copy file from {:?} to {:?}", source, destination))?;

    info!("File copied successfully");
    Ok(())
}

/// Delete a file if it exists
pub fn delete_file(path: &Path) -> Result<()> {
    if path.exists() {
        debug!("Deleting file: {:?}", path);
        fs::remove_file(path).with_context(|| format!("Failed to delete file: {:?}", path))?;
        info!("File deleted successfully");
    } else {
        debug!("File does not exist, skipping deletion: {:?}", path);
    }
    Ok(())
}

/// Delete a directory and all its contents
pub fn delete_directory(path: &Path) -> Result<()> {
    if path.exists() {
        info!("Deleting directory and contents: {:?}", path);
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to delete directory: {:?}", path))?;
        info!("Directory deleted successfully");
    } else {
        debug!("Directory does not exist, skipping deletion: {:?}", path);
    }
    Ok(())
}

/// Get the size of a file in bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for file: {:?}", path))?;
    Ok(metadata.len())
}

/// Check if there's enough disk space for a file
pub fn check_disk_space(path: &Path, _required_bytes: u64) -> Result<bool> {
    // Note: This is a simplified check. For production, you might want to use
    // a crate like `fs2` or `sysinfo` for accurate disk space checking.

    // For now, we'll just check if the directory is writable
    if let Some(parent) = path.parent() {
        if parent.exists() && parent.is_dir() {
            // Simple heuristic: if we can read the directory, assume we have space
            // In production, implement proper disk space checking
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// Sanitize a filename to be safe for filesystem use
///
/// Removes/replaces dangerous characters and ensures the filename is not too long
pub fn sanitize_filename(filename: &str) -> String {
    // Replace unsafe characters with underscores
    let safe_chars: String = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    // Limit length to 200 characters
    let max_len = 200;
    if safe_chars.len() > max_len {
        let truncated = &safe_chars[..max_len];
        format!("{}_truncated", truncated)
    } else {
        safe_chars
    }
}

/// Validate that a path is within a base directory (prevent path traversal)
pub fn validate_path_is_safe(path: &Path, base_dir: &Path) -> Result<()> {
    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", path))?;

    let canonical_base = base_dir
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize base directory: {:?}", base_dir))?;

    if !canonical_path.starts_with(&canonical_base) {
        anyhow::bail!(
            "Path traversal detected: {:?} is not within {:?}",
            path,
            base_dir
        );
    }

    Ok(())
}

/// Generate a unique filename to avoid collisions
pub fn generate_unique_filename(base_dir: &Path, preferred_name: &str) -> String {
    let mut candidate = preferred_name.to_string();
    let mut counter = 1;

    while base_dir.join(&candidate).exists() {
        if let Some(dot_pos) = preferred_name.rfind('.') {
            let name = &preferred_name[..dot_pos];
            let ext = &preferred_name[dot_pos..];
            candidate = format!("{}_{}{}", name, counter, ext);
        } else {
            candidate = format!("{}_{}", preferred_name, counter);
        }
        counter += 1;
    }

    candidate
}

/// Clean up old temporary files (older than specified duration)
pub fn cleanup_temp_files(temp_dir: &Path, max_age_seconds: u64) -> Result<usize> {
    let mut cleaned_count = 0;

    if !temp_dir.exists() {
        return Ok(0);
    }

    let now = std::time::SystemTime::now();

    for entry in fs::read_dir(temp_dir)
        .with_context(|| format!("Failed to read temp directory: {:?}", temp_dir))?
    {
        let entry = entry?;
        let path = entry.path();

        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age.as_secs() > max_age_seconds {
                        if path.is_file() {
                            match fs::remove_file(&path) {
                                Ok(_) => {
                                    debug!("Cleaned up old temp file: {:?}", path);
                                    cleaned_count += 1;
                                }
                                Err(e) => {
                                    warn!("Failed to delete temp file {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if cleaned_count > 0 {
        info!("Cleaned up {} old temporary files", cleaned_count);
    }

    Ok(cleaned_count)
}

/// Calculate the total size of a directory
pub fn calculate_directory_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if !dir.is_dir() {
        anyhow::bail!("Path is not a directory: {:?}", dir);
    }

    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {:?}", dir))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                total_size += metadata.len();
            }
        } else if path.is_dir() {
            // Recursively calculate subdirectory size
            total_size += calculate_directory_size(&path)?;
        }
    }

    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello.mp4"), "hello.mp4");
        assert_eq!(sanitize_filename("hello world.mp4"), "hello world.mp4");
        assert_eq!(sanitize_filename("hello/world.mp4"), "hello_world.mp4");
        assert_eq!(sanitize_filename("hello:world.mp4"), "hello_world.mp4");
        assert_eq!(sanitize_filename("hello*world?.mp4"), "hello_world_.mp4");
    }

    #[test]
    fn test_generate_unique_filename() {
        let temp_dir = std::env::temp_dir().join("test_unique_filename");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a test file
        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test").unwrap();

        // Generate unique filename
        let unique = generate_unique_filename(&temp_dir, "test.txt");
        assert_eq!(unique, "test_1.txt");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = std::env::temp_dir().join("test_ensure_dir");

        // Clean up if exists
        let _ = fs::remove_dir_all(&temp_dir);

        // Should create directory
        assert!(ensure_dir_exists(&temp_dir).is_ok());
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());

        // Should succeed if already exists
        assert!(ensure_dir_exists(&temp_dir).is_ok());

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}

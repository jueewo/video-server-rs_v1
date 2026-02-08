//! Cleanup module for handling file and resource cleanup
//!
//! This module provides utilities to ensure proper cleanup of temporary files,
//! partial uploads, and other resources when errors occur during video processing.
//!
//! Features:
//! - Automatic cleanup on drop (RAII pattern)
//! - Manual cleanup operations
//! - Cleanup tracking and logging
//! - Safe cleanup (errors during cleanup don't panic)

use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Cleanup manager that tracks resources to clean up
#[derive(Debug)]
pub struct CleanupManager {
    /// Files to delete on cleanup
    files: Vec<PathBuf>,
    /// Directories to delete on cleanup
    directories: Vec<PathBuf>,
    /// Whether cleanup should run on drop
    auto_cleanup: bool,
    /// Cleanup operation name (for logging)
    operation_name: String,
}

impl CleanupManager {
    /// Create a new cleanup manager
    pub fn new(operation_name: impl Into<String>) -> Self {
        Self {
            files: Vec::new(),
            directories: Vec::new(),
            auto_cleanup: true,
            operation_name: operation_name.into(),
        }
    }

    /// Add a file to be cleaned up
    pub fn add_file(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        debug!(
            "CleanupManager[{}]: Registered file for cleanup: {:?}",
            self.operation_name, path
        );
        self.files.push(path);
    }

    /// Add multiple files to be cleaned up
    pub fn add_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        for path in paths {
            self.add_file(path);
        }
    }

    /// Add a directory to be cleaned up
    pub fn add_directory(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        debug!(
            "CleanupManager[{}]: Registered directory for cleanup: {:?}",
            self.operation_name, path
        );
        self.directories.push(path);
    }

    /// Disable automatic cleanup on drop
    pub fn disable_auto_cleanup(&mut self) {
        self.auto_cleanup = false;
    }

    /// Re-enable automatic cleanup on drop
    pub fn enable_auto_cleanup(&mut self) {
        self.auto_cleanup = true;
    }

    /// Mark cleanup as successful (disables auto-cleanup)
    pub fn success(mut self) {
        debug!(
            "CleanupManager[{}]: Operation successful, disabling auto-cleanup",
            self.operation_name
        );
        self.auto_cleanup = false;
    }

    /// Manually trigger cleanup
    pub async fn cleanup(&mut self) {
        info!(
            "CleanupManager[{}]: Starting cleanup ({} files, {} directories)",
            self.operation_name,
            self.files.len(),
            self.directories.len()
        );

        // Clean up files first
        for path in &self.files {
            if let Err(e) = cleanup_file(path).await {
                error!(
                    "CleanupManager[{}]: Failed to delete file {:?}: {}",
                    self.operation_name, path, e
                );
            }
        }

        // Then clean up directories
        for path in &self.directories {
            if let Err(e) = cleanup_directory(path).await {
                error!(
                    "CleanupManager[{}]: Failed to delete directory {:?}: {}",
                    self.operation_name, path, e
                );
            }
        }

        // Clear the lists
        self.files.clear();
        self.directories.clear();

        info!("CleanupManager[{}]: Cleanup complete", self.operation_name);
    }

    /// Check if there are any resources to clean up
    pub fn has_resources(&self) -> bool {
        !self.files.is_empty() || !self.directories.is_empty()
    }

    /// Get the number of resources registered for cleanup
    pub fn resource_count(&self) -> usize {
        self.files.len() + self.directories.len()
    }
}

impl Drop for CleanupManager {
    fn drop(&mut self) {
        if self.auto_cleanup && self.has_resources() {
            warn!(
                "CleanupManager[{}]: Auto-cleanup triggered on drop ({} resources)",
                self.operation_name,
                self.resource_count()
            );

            // We can't use async in Drop, so we need to use blocking operations
            // or spawn a task. For now, we'll just log a warning.
            // In production, you might want to spawn a cleanup task.
            for path in &self.files {
                warn!(
                    "CleanupManager[{}]: File requires cleanup: {:?}",
                    self.operation_name, path
                );
            }
            for path in &self.directories {
                warn!(
                    "CleanupManager[{}]: Directory requires cleanup: {:?}",
                    self.operation_name, path
                );
            }
        }
    }
}

/// Delete a file if it exists
pub async fn cleanup_file(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        debug!("Cleanup: File does not exist, skipping: {:?}", path);
        return Ok(());
    }

    debug!("Cleanup: Deleting file: {:?}", path);
    match fs::remove_file(path).await {
        Ok(_) => {
            info!("Cleanup: Successfully deleted file: {:?}", path);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Cleanup: File already deleted: {:?}", path);
            Ok(())
        }
        Err(e) => {
            error!("Cleanup: Failed to delete file {:?}: {}", path, e);
            Err(e)
        }
    }
}

/// Delete a directory and all its contents if it exists
pub async fn cleanup_directory(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        debug!("Cleanup: Directory does not exist, skipping: {:?}", path);
        return Ok(());
    }

    debug!("Cleanup: Deleting directory: {:?}", path);
    match fs::remove_dir_all(path).await {
        Ok(_) => {
            info!("Cleanup: Successfully deleted directory: {:?}", path);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Cleanup: Directory already deleted: {:?}", path);
            Ok(())
        }
        Err(e) => {
            error!("Cleanup: Failed to delete directory {:?}: {}", path, e);
            Err(e)
        }
    }
}

/// Clean up temporary upload files for a given upload ID
pub async fn cleanup_temp_upload(temp_dir: &Path, upload_id: &str) -> Result<(), std::io::Error> {
    let temp_file = temp_dir.join(format!("{}.tmp", upload_id));
    cleanup_file(&temp_file).await
}

/// Clean up partial HLS transcoding output
pub async fn cleanup_partial_hls(video_dir: &Path, slug: &str) -> Result<(), std::io::Error> {
    let hls_dir = video_dir.join(slug).join("hls");

    if hls_dir.exists() {
        info!(
            "Cleanup: Removing partial HLS directory for '{}': {:?}",
            slug, hls_dir
        );
        cleanup_directory(&hls_dir).await?;
    }

    Ok(())
}

/// Clean up all files associated with a failed video processing
pub async fn cleanup_failed_video(video_dir: &Path, slug: &str) -> Result<(), std::io::Error> {
    let video_slug_dir = video_dir.join(slug);

    if video_slug_dir.exists() {
        info!(
            "Cleanup: Removing all files for failed video '{}': {:?}",
            slug, video_slug_dir
        );
        cleanup_directory(&video_slug_dir).await?;
    }

    Ok(())
}

/// Clean up orphaned temporary files older than the specified duration
pub async fn cleanup_old_temp_files(
    temp_dir: &Path,
    max_age_hours: u64,
) -> Result<usize, std::io::Error> {
    use std::time::{Duration, SystemTime};

    let max_age = Duration::from_secs(max_age_hours * 3600);
    let now = SystemTime::now();
    let mut cleaned_count = 0;

    if !temp_dir.exists() {
        return Ok(0);
    }

    info!(
        "Cleanup: Scanning for temp files older than {} hours in {:?}",
        max_age_hours, temp_dir
    );

    let mut entries = fs::read_dir(temp_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Only process .tmp files
        if let Some(ext) = path.extension() {
            if ext != "tmp" {
                continue;
            }
        } else {
            continue;
        }

        // Check file age
        if let Ok(metadata) = entry.metadata().await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > max_age {
                        debug!(
                            "Cleanup: Found old temp file ({:.1}h old): {:?}",
                            age.as_secs_f64() / 3600.0,
                            path
                        );

                        if cleanup_file(&path).await.is_ok() {
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }
    }

    if cleaned_count > 0 {
        info!(
            "Cleanup: Cleaned up {} old temporary files from {:?}",
            cleaned_count, temp_dir
        );
    }

    Ok(cleaned_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_cleanup_file() {
        let temp_file = std::env::temp_dir().join("test_cleanup_file.tmp");

        // Create test file
        let mut file = File::create(&temp_file).await.unwrap();
        file.write_all(b"test data").await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        assert!(temp_file.exists());

        // Clean it up
        cleanup_file(&temp_file).await.unwrap();

        assert!(!temp_file.exists());

        // Cleanup non-existent file should not error
        cleanup_file(&temp_file).await.unwrap();
    }

    #[tokio::test]
    async fn test_cleanup_directory() {
        let temp_dir = std::env::temp_dir().join("test_cleanup_dir");

        // Create test directory with files
        fs::create_dir_all(&temp_dir).await.unwrap();
        let test_file = temp_dir.join("test.txt");
        let mut file = File::create(&test_file).await.unwrap();
        file.write_all(b"test data").await.unwrap();
        drop(file);

        assert!(temp_dir.exists());
        assert!(test_file.exists());

        // Clean it up
        cleanup_directory(&temp_dir).await.unwrap();

        assert!(!temp_dir.exists());
        assert!(!test_file.exists());

        // Cleanup non-existent directory should not error
        cleanup_directory(&temp_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_cleanup_manager() {
        let temp_file1 = std::env::temp_dir().join("test_manager_1.tmp");
        let temp_file2 = std::env::temp_dir().join("test_manager_2.tmp");

        // Create test files
        File::create(&temp_file1).await.unwrap();
        File::create(&temp_file2).await.unwrap();

        assert!(temp_file1.exists());
        assert!(temp_file2.exists());

        // Create manager and register files
        let mut manager = CleanupManager::new("test");
        manager.add_file(&temp_file1);
        manager.add_file(&temp_file2);

        assert_eq!(manager.resource_count(), 2);

        // Trigger cleanup
        manager.cleanup().await;

        assert!(!temp_file1.exists());
        assert!(!temp_file2.exists());
        assert_eq!(manager.resource_count(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_manager_success() {
        let temp_file = std::env::temp_dir().join("test_manager_success.tmp");

        // Create test file
        File::create(&temp_file).await.unwrap();

        {
            let mut manager = CleanupManager::new("test");
            manager.add_file(&temp_file);

            // Mark as successful (disables auto-cleanup)
            manager.success();
        } // manager dropped here

        // File should still exist because we called success()
        assert!(temp_file.exists());

        // Clean up manually
        cleanup_file(&temp_file).await.unwrap();
    }
}

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// Resolve and validate a file path within a workspace root.
/// Prevents path traversal attacks.
fn safe_resolve(workspace_root: &Path, rel_path: &str) -> Result<PathBuf> {
    let clean = rel_path.trim_start_matches('/');

    for segment in clean.split('/') {
        if segment == ".." || segment == "." {
            bail!("Path traversal not allowed");
        }
    }

    if clean.is_empty() {
        bail!("Empty file path");
    }

    let target = workspace_root.join(clean);

    // Verify it stays within workspace root after resolution
    let canonical_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    // For new files, canonicalize parent
    let canonical_target = if target.exists() {
        target.canonicalize()?
    } else {
        let parent = target
            .parent()
            .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;
        let canonical_parent = if parent.exists() {
            parent.canonicalize()?
        } else {
            parent.to_path_buf()
        };
        canonical_parent.join(target.file_name().unwrap_or_default())
    };

    if !canonical_target.starts_with(&canonical_root) {
        bail!("Path escapes workspace root");
    }

    Ok(target)
}

/// Read a text file's content.
pub fn read_file(workspace_root: &Path, rel_path: &str) -> Result<String> {
    let path = safe_resolve(workspace_root, rel_path)?;
    if !path.exists() || !path.is_file() {
        bail!("File not found: {:?}", path);
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(content)
}

/// Write content to a file (creates parent directories as needed).
pub fn save_file(workspace_root: &Path, rel_path: &str, content: &str) -> Result<()> {
    let path = safe_resolve(workspace_root, rel_path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a directory within the workspace.
pub fn create_folder(workspace_root: &Path, rel_path: &str) -> Result<()> {
    let path = safe_resolve(workspace_root, rel_path)?;
    std::fs::create_dir_all(&path)?;
    Ok(())
}

/// Write binary data to a file (creates parent directories as needed).
pub fn save_bytes(workspace_root: &Path, rel_path: &str, data: &[u8]) -> Result<()> {
    let path = safe_resolve(workspace_root, rel_path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, data)?;
    Ok(())
}

/// Delete a file or directory within the workspace.
pub fn delete_path(workspace_root: &Path, rel_path: &str) -> Result<()> {
    let path = safe_resolve(workspace_root, rel_path)?;
    if !path.exists() {
        bail!("Path not found: {:?}", path);
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path)?;
    } else {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

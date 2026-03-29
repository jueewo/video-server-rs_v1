//! Agent memory — file-based persistent context per agent.
//!
//! Each agent has a `memory.md` file in its source folder. This is loaded
//! before the LLM call and updated after, based on `<memory>` blocks in
//! the agent's response.

use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Resolve the memory file path for an agent.
/// Uses the agent's source workspace + file path to find its folder.
fn memory_path(workspace_root: &Path, workspace_id: &str, source_file: &str) -> PathBuf {
    let agent_dir = workspace_root
        .join("storage/vaults")
        .join(workspace_id)
        .join("media/documents")
        .join(source_file)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            workspace_root
                .join("storage/vaults")
                .join(workspace_id)
                .join("media/documents")
        });
    agent_dir.join("memory.md")
}

/// Load memory.md for an agent. Returns empty string if no memory exists.
pub fn load_memory(
    workspace_root: &Path,
    workspace_id: Option<&str>,
    source_file: Option<&str>,
) -> String {
    let (Some(ws_id), Some(src)) = (workspace_id, source_file) else {
        debug!("No workspace/source for agent memory");
        return String::new();
    };

    let path = memory_path(workspace_root, ws_id, src);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            debug!(?path, len = content.len(), "Loaded agent memory");
            content
        }
        Err(_) => {
            debug!(?path, "No memory file found");
            String::new()
        }
    }
}

/// Extract `<memory>...</memory>` block from LLM response text.
/// Returns the inner content if found.
pub fn extract_memory_block(response: &str) -> Option<String> {
    let start_tag = "<memory>";
    let end_tag = "</memory>";

    let start = response.find(start_tag)?;
    let content_start = start + start_tag.len();
    let end = response[content_start..].find(end_tag)?;

    let content = response[content_start..content_start + end].trim().to_string();
    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

/// Save memory content to agent's memory.md file.
/// Appends new content below existing memory with a timestamp separator.
pub fn save_memory(
    workspace_root: &Path,
    workspace_id: Option<&str>,
    source_file: Option<&str>,
    content: &str,
) -> anyhow::Result<()> {
    let (Some(ws_id), Some(src)) = (workspace_id, source_file) else {
        warn!("Cannot save memory: no workspace/source for agent");
        return Ok(());
    };

    let path = memory_path(workspace_root, ws_id, src);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Read existing memory and append
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let new_content = if existing.is_empty() {
        format!("# Agent Memory\n\n## {timestamp}\n{content}\n")
    } else {
        format!("{existing}\n## {timestamp}\n{content}\n")
    };

    std::fs::write(&path, &new_content)?;
    debug!(?path, "Saved agent memory");
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_memory_from_response() {
        let response = "Here is my analysis.\n\n<memory>\nUser prefers concise reports.\nOrder #123 had issues with shipping.\n</memory>\n\nThat's all.";
        let mem = extract_memory_block(response).unwrap();
        assert!(mem.contains("User prefers concise reports"));
        assert!(mem.contains("Order #123"));
    }

    #[test]
    fn extract_no_memory_block() {
        let response = "Just a regular response with no memory tags.";
        assert!(extract_memory_block(response).is_none());
    }

    #[test]
    fn extract_empty_memory_block() {
        let response = "Response <memory></memory> done.";
        assert!(extract_memory_block(response).is_none());
    }

    #[test]
    fn save_and_load_memory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create the expected directory structure
        let ws_id = "test-ws";
        let source_file = "agents/my-agent.yaml";

        // First save
        save_memory(root, Some(ws_id), Some(source_file), "First memory entry").unwrap();
        let loaded = load_memory(root, Some(ws_id), Some(source_file));
        assert!(loaded.contains("First memory entry"));
        assert!(loaded.contains("# Agent Memory"));

        // Second save appends
        save_memory(root, Some(ws_id), Some(source_file), "Second entry").unwrap();
        let loaded = load_memory(root, Some(ws_id), Some(source_file));
        assert!(loaded.contains("First memory entry"));
        assert!(loaded.contains("Second entry"));
    }

    #[test]
    fn load_nonexistent_memory() {
        let loaded = load_memory(Path::new("/nonexistent"), Some("ws"), Some("agent.yaml"));
        assert!(loaded.is_empty());
    }
}

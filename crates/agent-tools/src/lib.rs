//! Agent Tools — domain-specific tool definitions for AI agents.
//!
//! These tools define the interface that ZeroClaw (or any agent runner) uses
//! to interact with workspace files. Each tool is described as a JSON schema
//! that the agent runner can present to the LLM, and a handler function that
//! the runner calls back into video-server-rs to execute.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ============================================================================
// Tool definitions (JSON Schema for LLM tool calling)
// ============================================================================

/// A tool definition that can be registered with an agent runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Returns all available workspace tools for agent registration.
pub fn workspace_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "workspace_read_file".to_string(),
            description: "Read the contents of a file in the workspace.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative file path to read"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "workspace_write_file".to_string(),
            description: "Write content to a file in the workspace. Creates the file if it doesn't exist.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative file path to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Full file content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: "workspace_list_files".to_string(),
            description: "List files and folders in a workspace directory.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative directory path (empty string for root)"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "workspace_search".to_string(),
            description: "Search for text content across files in the workspace.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Text to search for (case-insensitive)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (empty for workspace root)"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "folder_structure".to_string(),
            description: "Get the folder type information, key files, and structure description for a folder.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative folder path"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "workspace_context".to_string(),
            description: "Get the full workspace context: file listing, folder types, and project description.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
            }),
        },
    ]
}

// ============================================================================
// Tool execution (server-side handlers)
// ============================================================================

/// Result from executing a tool.
#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(output: serde_json::Value) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            error: Some(message.into()),
        }
    }
}

/// Resolve and validate a workspace-relative path, preventing path traversal.
pub fn safe_resolve(workspace_root: &Path, subpath: &str) -> Result<PathBuf> {
    let subpath = subpath.trim_start_matches('/');
    let resolved = workspace_root.join(subpath);
    let canonical_root = workspace_root
        .canonicalize()
        .context("Failed to canonicalize workspace root")?;

    // For new files that don't exist yet, check the parent
    let check_path = if resolved.exists() {
        resolved
            .canonicalize()
            .context("Failed to canonicalize resolved path")?
    } else {
        let parent = resolved
            .parent()
            .context("No parent directory")?;
        if !parent.exists() {
            anyhow::bail!("Parent directory does not exist");
        }
        let canonical_parent = parent.canonicalize()?;
        canonical_parent.join(resolved.file_name().context("No filename")?)
    };

    if !check_path.starts_with(&canonical_root) {
        anyhow::bail!("Path traversal detected");
    }

    Ok(resolved)
}

/// Execute `workspace_read_file` tool.
pub fn exec_read_file(workspace_root: &Path, path: &str) -> ToolResult {
    match safe_resolve(workspace_root, path) {
        Ok(resolved) => {
            if !resolved.is_file() {
                return ToolResult::err(format!("Not a file: {}", path));
            }
            match std::fs::read_to_string(&resolved) {
                Ok(content) => ToolResult::ok(serde_json::json!({
                    "path": path,
                    "content": content,
                    "size": content.len(),
                })),
                Err(e) => ToolResult::err(format!("Failed to read file: {}", e)),
            }
        }
        Err(e) => ToolResult::err(format!("Invalid path: {}", e)),
    }
}

/// Execute `workspace_write_file` tool.
pub fn exec_write_file(workspace_root: &Path, path: &str, content: &str) -> ToolResult {
    match safe_resolve(workspace_root, path) {
        Ok(resolved) => {
            // Create parent directories if needed
            if let Some(parent) = resolved.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        return ToolResult::err(format!("Failed to create directories: {}", e));
                    }
                }
            }
            match std::fs::write(&resolved, content) {
                Ok(()) => ToolResult::ok(serde_json::json!({
                    "path": path,
                    "bytes_written": content.len(),
                })),
                Err(e) => ToolResult::err(format!("Failed to write file: {}", e)),
            }
        }
        Err(e) => ToolResult::err(format!("Invalid path: {}", e)),
    }
}

/// Execute `workspace_list_files` tool.
pub fn exec_list_files(workspace_root: &Path, path: &str) -> ToolResult {
    match safe_resolve(workspace_root, path) {
        Ok(resolved) => {
            if !resolved.is_dir() {
                return ToolResult::err(format!("Not a directory: {}", path));
            }
            let mut folders = Vec::new();
            let mut files = Vec::new();

            match std::fs::read_dir(&resolved) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }
                        let meta = entry.metadata().ok();
                        if meta.as_ref().map_or(false, |m| m.is_dir()) {
                            folders.push(name);
                        } else {
                            let size = meta.as_ref().map_or(0, |m| m.len());
                            files.push(serde_json::json!({
                                "name": name,
                                "size": size,
                            }));
                        }
                    }
                    folders.sort();
                    files.sort_by(|a, b| {
                        a["name"].as_str().cmp(&b["name"].as_str())
                    });
                    ToolResult::ok(serde_json::json!({
                        "path": path,
                        "folders": folders,
                        "files": files,
                    }))
                }
                Err(e) => ToolResult::err(format!("Failed to read directory: {}", e)),
            }
        }
        Err(e) => ToolResult::err(format!("Invalid path: {}", e)),
    }
}

/// Execute `workspace_search` tool.
pub fn exec_search(workspace_root: &Path, query: &str, path: &str) -> ToolResult {
    let search_root = match safe_resolve(workspace_root, path) {
        Ok(r) => r,
        Err(e) => return ToolResult::err(format!("Invalid path: {}", e)),
    };

    if !search_root.is_dir() {
        return ToolResult::err(format!("Not a directory: {}", path));
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    let max_results = 50;
    let max_file_size = 100_000u64;

    fn walk_search(
        dir: &Path,
        root: &Path,
        query: &str,
        matches: &mut Vec<serde_json::Value>,
        max_results: usize,
        max_file_size: u64,
        depth: usize,
    ) {
        if depth > 10 || matches.len() >= max_results {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            if matches.len() >= max_results {
                break;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                walk_search(&path, root, query, matches, max_results, max_file_size, depth + 1);
            } else if path.is_file() {
                let meta = entry.metadata().ok();
                let size = meta.as_ref().map_or(0, |m| m.len());
                if size > max_file_size || size == 0 {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let rel = path.strip_prefix(root).unwrap_or(&path);
                    for (line_no, line) in content.lines().enumerate() {
                        if matches.len() >= max_results {
                            break;
                        }
                        if line.to_lowercase().contains(query) {
                            matches.push(serde_json::json!({
                                "file": rel.to_string_lossy(),
                                "line": line_no + 1,
                                "content": line.trim(),
                            }));
                        }
                    }
                }
            }
        }
    }

    walk_search(
        &search_root,
        workspace_root,
        &query_lower,
        &mut matches,
        max_results,
        max_file_size,
    0,
    );

    ToolResult::ok(serde_json::json!({
        "query": query,
        "matches": matches,
        "total": matches.len(),
    }))
}

/// Dispatch a tool call by name.
pub fn dispatch_tool(
    workspace_root: &Path,
    tool_name: &str,
    params: &serde_json::Value,
) -> ToolResult {
    match tool_name {
        "workspace_read_file" => {
            let path = params["path"].as_str().unwrap_or("");
            exec_read_file(workspace_root, path)
        }
        "workspace_write_file" => {
            let path = params["path"].as_str().unwrap_or("");
            let content = params["content"].as_str().unwrap_or("");
            exec_write_file(workspace_root, path, content)
        }
        "workspace_list_files" => {
            let path = params["path"].as_str().unwrap_or("");
            exec_list_files(workspace_root, path)
        }
        "workspace_search" => {
            let query = params["query"].as_str().unwrap_or("");
            let path = params["path"].as_str().unwrap_or("");
            exec_search(workspace_root, query, path)
        }
        _ => ToolResult::err(format!("Unknown tool: {}", tool_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        let tools = workspace_tools();
        assert!(tools.len() >= 6);
        assert!(tools.iter().any(|t| t.name == "workspace_read_file"));
        assert!(tools.iter().any(|t| t.name == "workspace_write_file"));
        assert!(tools.iter().any(|t| t.name == "workspace_list_files"));
        assert!(tools.iter().any(|t| t.name == "workspace_search"));
    }

    #[test]
    fn test_read_write_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();

        // Write
        let result = exec_write_file(root, "test.txt", "hello world");
        assert!(result.success);

        // Read back
        let result = exec_read_file(root, "test.txt");
        assert!(result.success);
        assert_eq!(result.output["content"], "hello world");
    }

    #[test]
    fn test_list_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();

        std::fs::write(root.join("a.txt"), "aaa").unwrap();
        std::fs::write(root.join("b.txt"), "bbb").unwrap();
        std::fs::create_dir(root.join("subdir")).unwrap();

        let result = exec_list_files(root, "");
        assert!(result.success);
        assert_eq!(result.output["folders"][0], "subdir");
        assert_eq!(result.output["files"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_search() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();

        std::fs::write(root.join("a.txt"), "Hello World\nGoodbye World").unwrap();
        std::fs::write(root.join("b.txt"), "No match here").unwrap();

        let result = exec_search(root, "goodbye", "");
        assert!(result.success);
        assert_eq!(result.output["total"], 1);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();

        let result = exec_read_file(root, "../../../etc/passwd");
        assert!(!result.success);
    }

    #[test]
    fn test_dispatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();

        let result = dispatch_tool(
            root,
            "workspace_write_file",
            &serde_json::json!({"path": "hello.md", "content": "# Hello"}),
        );
        assert!(result.success);

        let result = dispatch_tool(
            root,
            "workspace_read_file",
            &serde_json::json!({"path": "hello.md"}),
        );
        assert!(result.success);
        assert_eq!(result.output["content"], "# Hello");
    }
}

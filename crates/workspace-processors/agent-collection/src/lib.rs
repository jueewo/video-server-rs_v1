//! Agent Collection Processor
//!
//! Manages collections of AI agents defined in markdown files.
//! Agents can be loaded into ZeroClaw, Claude Code, API integrations, or custom workflows.
//!
//! ## Agent File Format (Markdown with YAML frontmatter)
//! ```markdown
//! ---
//! role: content-writer
//! model: claude-sonnet-4.5
//! tools: [read_file, write_file, list_files]
//! temperature: 0.7
//! folder_types: [static-site, course]
//! autonomy: supervised
//! ---
//!
//! # Content Writer Agent
//!
//! You create and edit content following structure conventions...
//! ```
//!
//! ## Configuration (workspace.yaml)
//! ```yaml
//! folders:
//!   "agents":
//!     type: agent-collection
//!     agents:
//!       - file: coder.md
//!         role: code-generation
//!         model: claude-sonnet-4.5
//!       - file: reviewer.md
//!         role: code-review
//!         model: claude-opus-4.6
//!     shared_context: context/project-docs.md
//!     memory_dir: .memory/
//! ```

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Configuration for an agent collection
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AgentCollectionConfig {
    pub agents: Vec<AgentRef>,
    pub shared_context: Option<String>,
    pub memory_dir: Option<String>,
}

/// Reference to an agent file
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AgentRef {
    pub file: String,
    pub role: Option<String>,
    pub model: Option<String>,
}

/// YAML frontmatter parsed from agent markdown files.
#[derive(Debug, Clone, serde::Deserialize)]
struct AgentFrontmatter {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    folder_types: Vec<String>,
    #[serde(default = "default_autonomy")]
    autonomy: String,
    #[serde(default, flatten)]
    extra: HashMap<String, serde_yaml::Value>,
}

fn default_autonomy() -> String {
    "supervised".to_string()
}

/// Parsed agent definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub role: String,
    pub model: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub temperature: f32,
    /// Folder types this agent is compatible with (empty = all types).
    #[serde(default)]
    pub folder_types: Vec<String>,
    /// Autonomy level: "autonomous", "supervised", or "manual".
    #[serde(default = "default_autonomy")]
    pub autonomy: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
    pub system_prompt: String,
}

impl Default for AgentDefinition {
    fn default() -> Self {
        Self {
            name: "agent".to_string(),
            role: "assistant".to_string(),
            model: "claude-sonnet-4.5".to_string(),
            tools: vec![],
            temperature: 1.0,
            folder_types: vec![],
            autonomy: "supervised".to_string(),
            metadata: HashMap::new(),
            system_prompt: String::new(),
        }
    }
}

/// Load an agent definition from a markdown file with YAML frontmatter.
pub fn load_agent(file_path: &Path) -> Result<AgentDefinition> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read agent file {:?}", file_path))?;

    let (frontmatter, body) = parse_frontmatter(&content)
        .with_context(|| format!("Failed to parse frontmatter in {:?}", file_path))?;

    // Derive name from filename (without extension)
    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("agent")
        .to_string();

    Ok(AgentDefinition {
        name,
        role: frontmatter.role.unwrap_or_else(|| "assistant".to_string()),
        model: frontmatter.model.unwrap_or_else(|| "claude-sonnet-4.5".to_string()),
        tools: frontmatter.tools,
        temperature: frontmatter.temperature.unwrap_or(1.0),
        folder_types: frontmatter.folder_types,
        autonomy: frontmatter.autonomy,
        metadata: frontmatter.extra,
        system_prompt: body.trim().to_string(),
    })
}

/// Load all agents in a collection.
///
/// If `config` has an explicit agent list, only those files are loaded.
/// Otherwise, all `*.md` files in the folder are loaded.
pub fn load_collection(folder_path: &Path, config: &AgentCollectionConfig) -> Result<Vec<AgentDefinition>> {
    let mut agents = Vec::new();

    if config.agents.is_empty() {
        // Auto-discover: load all .md files in the folder
        if let Ok(entries) = std::fs::read_dir(folder_path) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
                .collect();
            paths.sort();

            for path in paths {
                match load_agent(&path) {
                    Ok(agent) => agents.push(agent),
                    Err(e) => {
                        tracing::warn!("Failed to load agent {:?}: {}", path, e);
                    }
                }
            }
        }
    } else {
        for agent_ref in &config.agents {
            let agent_path = folder_path.join(&agent_ref.file);
            match load_agent(&agent_path) {
                Ok(mut agent) => {
                    // Override with config if specified
                    if let Some(role) = &agent_ref.role {
                        agent.role = role.clone();
                    }
                    if let Some(model) = &agent_ref.model {
                        agent.model = model.clone();
                    }
                    agents.push(agent);
                }
                Err(e) => {
                    tracing::warn!("Failed to load agent {}: {}", agent_ref.file, e);
                }
            }
        }
    }

    Ok(agents)
}

/// Auto-discover and load all agent markdown files from a folder (no config needed).
pub fn discover_agents(folder_path: &Path) -> Result<Vec<AgentDefinition>> {
    let config = AgentCollectionConfig {
        agents: vec![],
        shared_context: None,
        memory_dir: None,
    };
    load_collection(folder_path, &config)
}

/// Export agents in ZeroClaw-compatible configuration format.
///
/// Returns a JSON value that can be sent to ZeroClaw as agent session config.
pub fn export_for_zeroclaw(agents: &[AgentDefinition]) -> Result<serde_json::Value> {
    let agent_configs: Vec<serde_json::Value> = agents
        .iter()
        .map(|agent| {
            serde_json::json!({
                "name": agent.name,
                "role": agent.role,
                "model": agent.model,
                "system_prompt": agent.system_prompt,
                "tools": agent.tools,
                "temperature": agent.temperature,
                "autonomy": agent.autonomy,
                "folder_types": agent.folder_types,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "agents": agent_configs,
    }))
}

/// Export agents for Claude Code CLI integration.
pub fn export_for_claude_code(agents: &[AgentDefinition]) -> Result<String> {
    let configs: Vec<serde_json::Value> = agents
        .iter()
        .map(|agent| {
            serde_json::json!({
                "name": agent.name,
                "role": agent.role,
                "model": agent.model,
                "system_prompt": agent.system_prompt,
                "tools": agent.tools,
                "temperature": agent.temperature,
            })
        })
        .collect();

    serde_json::to_string_pretty(&configs).context("Failed to serialize agents for Claude Code")
}

/// Export agents for Claude API integration.
pub fn export_for_api(agents: &[AgentDefinition]) -> Result<Vec<serde_json::Value>> {
    Ok(agents
        .iter()
        .map(|agent| {
            serde_json::json!({
                "model": agent.model,
                "system": agent.system_prompt,
                "temperature": agent.temperature,
                "max_tokens": 4096,
            })
        })
        .collect())
}

// ============================================================================
// Frontmatter parser
// ============================================================================

/// Parse YAML frontmatter from a markdown string.
///
/// Expects the document to start with `---\n` and have a closing `---\n`.
/// Returns the parsed frontmatter and the remaining markdown body.
fn parse_frontmatter(content: &str) -> Result<(AgentFrontmatter, String)> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        // No frontmatter — return defaults and entire content as body
        return Ok((
            AgentFrontmatter {
                role: None,
                model: None,
                tools: vec![],
                temperature: None,
                folder_types: vec![],
                autonomy: default_autonomy(),
                extra: HashMap::new(),
            },
            content.to_string(),
        ));
    }

    // Find the closing ---
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.trim_start_matches(['\r', '\n']);

    let closing_pos = after_opening
        .find("\n---")
        .context("No closing --- found for frontmatter")?;

    let yaml_str = &after_opening[..closing_pos];
    let body_start = closing_pos + 4; // skip "\n---"
    let body = if body_start < after_opening.len() {
        after_opening[body_start..].trim_start_matches(['\r', '\n'])
    } else {
        ""
    };

    let frontmatter: AgentFrontmatter =
        serde_yaml::from_str(yaml_str).context("Failed to parse YAML frontmatter")?;

    Ok((frontmatter, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_config_parsing() {
        let yaml = r#"
agents:
  - file: coder.md
    role: code-generation
    model: claude-sonnet-4.5
  - file: reviewer.md
    role: code-review
shared_context: context/docs.md
"#;
        let config: AgentCollectionConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agents.len(), 2);
        assert_eq!(config.shared_context, Some("context/docs.md".to_string()));
    }

    #[test]
    fn test_agent_default() {
        let agent = AgentDefinition::default();
        assert_eq!(agent.model, "claude-sonnet-4.5");
        assert_eq!(agent.temperature, 1.0);
        assert_eq!(agent.autonomy, "supervised");
        assert!(agent.folder_types.is_empty());
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
role: content-writer
model: claude-sonnet-4.5
tools: [read_file, write_file]
temperature: 0.7
folder_types: [static-site, course]
autonomy: supervised
---

# Content Writer Agent

You create and edit content.
"#;
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.role, Some("content-writer".to_string()));
        assert_eq!(fm.model, Some("claude-sonnet-4.5".to_string()));
        assert_eq!(fm.tools, vec!["read_file", "write_file"]);
        assert_eq!(fm.temperature, Some(0.7));
        assert_eq!(fm.folder_types, vec!["static-site", "course"]);
        assert_eq!(fm.autonomy, "supervised");
        assert!(body.contains("# Content Writer Agent"));
        assert!(body.contains("You create and edit content."));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a markdown file\n\nWith some content.";
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert!(fm.role.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_load_agent_from_file() {
        let dir = TempDir::new().unwrap();
        let agent_path = dir.path().join("writer.md");
        let mut f = std::fs::File::create(&agent_path).unwrap();
        write!(
            f,
            r#"---
role: content-writer
model: claude-sonnet-4.5
tools: [read_file, write_file]
folder_types: [static-site]
---

# Writer
You write content.
"#
        )
        .unwrap();

        let agent = load_agent(&agent_path).unwrap();
        assert_eq!(agent.name, "writer");
        assert_eq!(agent.role, "content-writer");
        assert_eq!(agent.model, "claude-sonnet-4.5");
        assert_eq!(agent.tools, vec!["read_file", "write_file"]);
        assert_eq!(agent.folder_types, vec!["static-site"]);
        assert!(agent.system_prompt.contains("# Writer"));
    }

    #[test]
    fn test_discover_agents() {
        let dir = TempDir::new().unwrap();

        // Create two agent files
        std::fs::write(
            dir.path().join("alpha.md"),
            "---\nrole: alpha\n---\n# Alpha Agent",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("beta.md"),
            "---\nrole: beta\n---\n# Beta Agent",
        )
        .unwrap();
        // Non-md file should be ignored
        std::fs::write(dir.path().join("readme.txt"), "not an agent").unwrap();

        let agents = discover_agents(dir.path()).unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "alpha");
        assert_eq!(agents[1].name, "beta");
    }

    #[test]
    fn test_export_for_zeroclaw() {
        let agents = vec![AgentDefinition {
            name: "test".to_string(),
            role: "writer".to_string(),
            model: "claude-sonnet-4.5".to_string(),
            tools: vec!["read_file".to_string()],
            temperature: 0.7,
            folder_types: vec!["static-site".to_string()],
            autonomy: "supervised".to_string(),
            metadata: HashMap::new(),
            system_prompt: "You are a test agent.".to_string(),
        }];

        let result = export_for_zeroclaw(&agents).unwrap();
        assert!(result["agents"].is_array());
        assert_eq!(result["agents"][0]["name"], "test");
        assert_eq!(result["agents"][0]["autonomy"], "supervised");
    }
}

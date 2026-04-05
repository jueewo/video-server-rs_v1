//! Agent Collection Processor
//!
//! Manages collections of AI agents defined in markdown or YAML files.
//! Agents can be loaded into ZeroClaw, Claude Code, API integrations, or custom workflows.
//!
//! ## Agent File Format — YAML (recommended)
//! ```yaml
//! role: content-writer
//! model: claude-sonnet-4.5
//! tools: [workspace_read_file, workspace_write_file, workspace_list_files]
//! temperature: 0.7
//! folder_types: [static-site, course]
//! autonomy: supervised
//! system_prompt: |
//!   # Content Writer Agent
//!
//!   You create and edit content following structure conventions...
//! ```
//!
//! ## Agent File Format — Markdown (with YAML frontmatter)
//! ```markdown
//! ---
//! role: content-writer
//! model: claude-sonnet-4.5
//! tools: [workspace_read_file, workspace_write_file, workspace_list_files]
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
//!       - file: coder.yaml
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

// ============================================================================
// Known valid values (for validation)
// ============================================================================

/// Valid tool names that agents can use.
const VALID_TOOLS: &[&str] = &[
    "workspace_read_file",
    "workspace_write_file",
    "workspace_list_files",
    "workspace_search",
    "folder_structure",
    "workspace_context",
];

/// Valid autonomy levels.
const VALID_AUTONOMY: &[&str] = &["autonomous", "supervised", "manual"];

// ============================================================================
// Data types
// ============================================================================

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

/// Agent definition parsed from YAML or TOML (also used for .md frontmatter).
#[derive(Debug, Clone, serde::Deserialize)]
struct AgentYaml {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    description: Option<String>,
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
    /// Max tool-use iterations before the agent stops (default: 10).
    #[serde(default)]
    max_iterations: Option<u32>,
    /// Max output tokens per LLM call.
    #[serde(default)]
    max_tokens: Option<u32>,
    /// Execution timeout in seconds (default: 300). Prevents runaway agents.
    #[serde(default)]
    timeout: Option<u32>,
    /// Max delegation depth — prevents infinite agent-calling-agent loops (default: 3).
    #[serde(default)]
    max_depth: Option<u32>,
    /// System prompt — used in .yaml/.toml files (in .md files the body is the prompt).
    #[serde(default)]
    system_prompt: Option<String>,
    #[serde(default, flatten)]
    extra: HashMap<String, serde_yaml::Value>,
}

/// TOML-specific agent struct (toml crate doesn't support serde_yaml::Value in flatten).
#[derive(Debug, Clone, serde::Deserialize)]
struct AgentToml {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    description: Option<String>,
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
    #[serde(default)]
    max_iterations: Option<u32>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    timeout: Option<u32>,
    #[serde(default)]
    max_depth: Option<u32>,
    #[serde(default)]
    system_prompt: Option<String>,
}

fn default_autonomy() -> String {
    "supervised".to_string()
}

/// A single validation problem found in an agent definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Parsed agent definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub role: String,
    /// Human-readable description (shown in UI, separate from system prompt).
    #[serde(default)]
    pub description: String,
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
    /// Max tool-use iterations before the agent stops. Default: 10.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// Max output tokens per LLM call. Default: 4096.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Execution timeout in seconds. Prevents runaway agents. Default: 300.
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    /// Max delegation depth — prevents infinite agent-calling-agent loops. Default: 3.
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
    pub system_prompt: String,
    /// Source file format: "yaml", "toml", or "md".
    #[serde(default = "default_format")]
    pub format: String,
    /// Whether this agent passed validation. Invalid agents are not offered in the agent panel.
    #[serde(default = "default_true")]
    pub active: bool,
    /// Validation errors (empty if valid).
    #[serde(default)]
    pub validation_errors: Vec<ValidationError>,
}

fn default_max_iterations() -> u32 {
    10
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_timeout() -> u32 {
    300
}

fn default_max_depth() -> u32 {
    3
}

fn default_format() -> String {
    "md".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for AgentDefinition {
    fn default() -> Self {
        Self {
            name: "agent".to_string(),
            role: "assistant".to_string(),
            description: String::new(),
            model: "claude-sonnet-4.5".to_string(),
            tools: vec![],
            temperature: 1.0,
            folder_types: vec![],
            autonomy: "supervised".to_string(),
            max_iterations: 10,
            max_tokens: 4096,
            timeout: 300,
            max_depth: 3,
            metadata: HashMap::new(),
            system_prompt: String::new(),
            format: "md".to_string(),
            active: true,
            validation_errors: vec![],
        }
    }
}

// ============================================================================
// Validation
// ============================================================================

/// Validate an agent definition and return any errors found.
pub fn validate_agent(agent: &AgentDefinition) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Role must be explicitly set (not the default "assistant" fallback)
    if agent.role.trim().is_empty() || agent.role == "assistant" {
        errors.push(ValidationError {
            field: "role".to_string(),
            message: "Role must be explicitly set (e.g. content-writer, course-planner)".to_string(),
        });
    }

    // Model must be non-empty
    if agent.model.trim().is_empty() {
        errors.push(ValidationError {
            field: "model".to_string(),
            message: "Model must not be empty".to_string(),
        });
    }

    // At least one tool required
    if agent.tools.is_empty() {
        errors.push(ValidationError {
            field: "tools".to_string(),
            message: "At least one tool must be specified".to_string(),
        });
    }

    // Validate tool names
    for tool in &agent.tools {
        if !VALID_TOOLS.contains(&tool.as_str()) {
            errors.push(ValidationError {
                field: "tools".to_string(),
                message: format!(
                    "Unknown tool '{}'. Valid tools: {}",
                    tool,
                    VALID_TOOLS.join(", ")
                ),
            });
        }
    }

    // Validate autonomy
    if !VALID_AUTONOMY.contains(&agent.autonomy.as_str()) {
        errors.push(ValidationError {
            field: "autonomy".to_string(),
            message: format!(
                "Invalid autonomy '{}'. Must be one of: {}",
                agent.autonomy,
                VALID_AUTONOMY.join(", ")
            ),
        });
    }

    // Temperature range
    if agent.temperature < 0.0 || agent.temperature > 2.0 {
        errors.push(ValidationError {
            field: "temperature".to_string(),
            message: format!(
                "Temperature {} is out of range (0.0 – 2.0)",
                agent.temperature
            ),
        });
    }

    // Timeout must be reasonable (1 second to 1 hour)
    if agent.timeout == 0 || agent.timeout > 3600 {
        errors.push(ValidationError {
            field: "timeout".to_string(),
            message: format!(
                "Timeout {} is out of range (1 – 3600 seconds)",
                agent.timeout
            ),
        });
    }

    // Max depth must be at least 1
    if agent.max_depth == 0 || agent.max_depth > 20 {
        errors.push(ValidationError {
            field: "max_depth".to_string(),
            message: format!(
                "Max depth {} is out of range (1 – 20)",
                agent.max_depth
            ),
        });
    }

    // System prompt should not be empty
    if agent.system_prompt.trim().is_empty() {
        errors.push(ValidationError {
            field: "system_prompt".to_string(),
            message: "System prompt is empty".to_string(),
        });
    }

    errors
}

// ============================================================================
// Loading
// ============================================================================

/// Load an agent definition from a file (.md or .yaml/.yml).
///
/// The agent is always returned (even if invalid) with `active` and
/// `validation_errors` set accordingly.
pub fn load_agent(file_path: &Path) -> Result<AgentDefinition> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut agent = match ext.as_str() {
        "yaml" | "yml" => load_agent_yaml(file_path)?,
        "toml" => load_agent_toml(file_path)?,
        "md" | "markdown" => load_agent_md(file_path)?,
        _ => anyhow::bail!("Unsupported agent file extension: .{ext}"),
    };

    // Validate and set active status
    let errors = validate_agent(&agent);
    agent.active = errors.is_empty();
    agent.validation_errors = errors;

    Ok(agent)
}

/// Load an agent from a YAML file.
fn load_agent_yaml(file_path: &Path) -> Result<AgentDefinition> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read agent file {:?}", file_path))?;

    let parsed: AgentYaml =
        serde_yaml::from_str(&content).with_context(|| format!("Failed to parse YAML in {:?}", file_path))?;

    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("agent")
        .to_string();

    Ok(AgentDefinition {
        name,
        role: parsed.role.unwrap_or_else(|| "assistant".to_string()),
        description: parsed.description.unwrap_or_default(),
        model: parsed.model.unwrap_or_else(|| "claude-sonnet-4.5".to_string()),
        tools: parsed.tools,
        temperature: parsed.temperature.unwrap_or(1.0),
        folder_types: parsed.folder_types,
        autonomy: parsed.autonomy,
        max_iterations: parsed.max_iterations.unwrap_or(10),
        max_tokens: parsed.max_tokens.unwrap_or(4096),
        timeout: parsed.timeout.unwrap_or(300),
        max_depth: parsed.max_depth.unwrap_or(3),
        metadata: parsed.extra,
        system_prompt: parsed.system_prompt.unwrap_or_default().trim().to_string(),
        format: "yaml".to_string(),
        active: true,
        validation_errors: vec![],
    })
}

/// Load an agent from a TOML file.
fn load_agent_toml(file_path: &Path) -> Result<AgentDefinition> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read agent file {:?}", file_path))?;

    let parsed: AgentToml =
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML in {:?}", file_path))?;

    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("agent")
        .to_string();

    Ok(AgentDefinition {
        name,
        role: parsed.role.unwrap_or_else(|| "assistant".to_string()),
        description: parsed.description.unwrap_or_default(),
        model: parsed.model.unwrap_or_else(|| "claude-sonnet-4.5".to_string()),
        tools: parsed.tools,
        temperature: parsed.temperature.unwrap_or(1.0),
        folder_types: parsed.folder_types,
        autonomy: parsed.autonomy,
        max_iterations: parsed.max_iterations.unwrap_or(10),
        max_tokens: parsed.max_tokens.unwrap_or(4096),
        timeout: parsed.timeout.unwrap_or(300),
        max_depth: parsed.max_depth.unwrap_or(3),
        metadata: HashMap::new(),
        system_prompt: parsed.system_prompt.unwrap_or_default().trim().to_string(),
        format: "toml".to_string(),
        active: true,
        validation_errors: vec![],
    })
}

/// Load an agent from a markdown file with YAML frontmatter.
fn load_agent_md(file_path: &Path) -> Result<AgentDefinition> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read agent file {:?}", file_path))?;

    let (frontmatter, body) = parse_frontmatter(&content)
        .with_context(|| format!("Failed to parse frontmatter in {:?}", file_path))?;

    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("agent")
        .to_string();

    Ok(AgentDefinition {
        name,
        role: frontmatter.role.unwrap_or_else(|| "assistant".to_string()),
        description: frontmatter.description.unwrap_or_default(),
        model: frontmatter.model.unwrap_or_else(|| "claude-sonnet-4.5".to_string()),
        tools: frontmatter.tools,
        temperature: frontmatter.temperature.unwrap_or(1.0),
        folder_types: frontmatter.folder_types,
        autonomy: frontmatter.autonomy,
        max_iterations: frontmatter.max_iterations.unwrap_or(10),
        max_tokens: frontmatter.max_tokens.unwrap_or(4096),
        timeout: frontmatter.timeout.unwrap_or(300),
        max_depth: frontmatter.max_depth.unwrap_or(3),
        metadata: frontmatter.extra,
        system_prompt: body.trim().to_string(),
        format: "md".to_string(),
        active: true,
        validation_errors: vec![],
    })
}

/// Load all agents in a collection.
///
/// If `config` has an explicit agent list, only those files are loaded.
/// Otherwise, all `*.md` and `*.yaml`/`*.yml` agent files in the folder are loaded.
/// Invalid agents are included but marked as `active: false`.
pub fn load_collection(folder_path: &Path, config: &AgentCollectionConfig) -> Result<Vec<AgentDefinition>> {
    let mut agents = Vec::new();

    if config.agents.is_empty() {
        // Auto-discover: load all .md and .yaml/.yml files in the folder
        if let Ok(entries) = std::fs::read_dir(folder_path) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| matches!(e, "md" | "markdown" | "yaml" | "yml" | "toml"))
                        .unwrap_or(false)
                })
                .collect();
            paths.sort();

            for path in paths {
                match load_agent(&path) {
                    Ok(agent) => {
                        if !agent.active {
                            tracing::warn!(
                                "Agent {:?} has validation errors: {:?}",
                                path,
                                agent.validation_errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
                            );
                        }
                        agents.push(agent);
                    }
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
                    // Re-validate after overrides
                    let errors = validate_agent(&agent);
                    agent.active = errors.is_empty();
                    agent.validation_errors = errors;
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

/// Auto-discover and load all agent files from a folder (no config needed).
pub fn discover_agents(folder_path: &Path) -> Result<Vec<AgentDefinition>> {
    let config = AgentCollectionConfig {
        agents: vec![],
        shared_context: None,
        memory_dir: None,
    };
    load_collection(folder_path, &config)
}

/// Return only active (valid) agents from a list.
pub fn active_agents(agents: &[AgentDefinition]) -> Vec<&AgentDefinition> {
    agents.iter().filter(|a| a.active).collect()
}

// ============================================================================
// Export
// ============================================================================

/// Export agents in ZeroClaw-compatible configuration format.
///
/// Only exports active agents. Returns a JSON value that can be sent to ZeroClaw.
pub fn export_for_zeroclaw(agents: &[AgentDefinition]) -> Result<serde_json::Value> {
    let agent_configs: Vec<serde_json::Value> = agents
        .iter()
        .filter(|a| a.active)
        .map(|agent| {
            serde_json::json!({
                "name": agent.name,
                "role": agent.role,
                "description": agent.description,
                "model": agent.model,
                "system_prompt": agent.system_prompt,
                "tools": agent.tools,
                "temperature": agent.temperature,
                "autonomy": agent.autonomy,
                "folder_types": agent.folder_types,
                "max_iterations": agent.max_iterations,
                "max_tokens": agent.max_tokens,
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
        .filter(|a| a.active)
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
        .filter(|a| a.active)
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
// Frontmatter parser (for .md files)
// ============================================================================

/// Parse YAML frontmatter from a markdown string.
///
/// Expects the document to start with `---\n` and have a closing `---\n`.
/// Returns the parsed frontmatter and the remaining markdown body.
fn parse_frontmatter(content: &str) -> Result<(AgentYaml, String)> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        // No frontmatter — return defaults and entire content as body
        return Ok((
            AgentYaml {
                role: None,
                description: None,
                model: None,
                tools: vec![],
                temperature: None,
                folder_types: vec![],
                autonomy: default_autonomy(),
                max_iterations: None,
                max_tokens: None,
                timeout: None,
                max_depth: None,
                system_prompt: None,
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

    let frontmatter: AgentYaml =
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
  - file: coder.yaml
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
        assert!(agent.active);
        assert!(agent.validation_errors.is_empty());
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
role: content-writer
model: claude-sonnet-4.5
tools: [workspace_read_file, workspace_write_file]
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
        assert_eq!(fm.tools, vec!["workspace_read_file", "workspace_write_file"]);
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
    fn test_load_agent_from_md() {
        let dir = TempDir::new().unwrap();
        let agent_path = dir.path().join("writer.md");
        let mut f = std::fs::File::create(&agent_path).unwrap();
        write!(
            f,
            r#"---
role: content-writer
model: claude-sonnet-4.5
tools: [workspace_read_file, workspace_write_file]
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
        assert_eq!(agent.tools, vec!["workspace_read_file", "workspace_write_file"]);
        assert_eq!(agent.folder_types, vec!["static-site"]);
        assert_eq!(agent.format, "md");
        assert!(agent.active);
        assert!(agent.system_prompt.contains("# Writer"));
    }

    #[test]
    fn test_load_agent_from_yaml() {
        let dir = TempDir::new().unwrap();
        let agent_path = dir.path().join("reviewer.yaml");
        std::fs::write(
            &agent_path,
            r#"
role: code-reviewer
model: claude-opus-4.6
tools: [workspace_read_file, workspace_list_files]
temperature: 0.3
folder_types: [documentation]
autonomy: manual
system_prompt: |
  # Code Reviewer

  You review code for quality and correctness.
"#,
        )
        .unwrap();

        let agent = load_agent(&agent_path).unwrap();
        assert_eq!(agent.name, "reviewer");
        assert_eq!(agent.role, "code-reviewer");
        assert_eq!(agent.model, "claude-opus-4.6");
        assert_eq!(agent.temperature, 0.3);
        assert_eq!(agent.autonomy, "manual");
        assert_eq!(agent.format, "yaml");
        assert!(agent.active);
        assert!(agent.system_prompt.contains("# Code Reviewer"));
    }

    #[test]
    fn test_load_agent_from_toml() {
        let dir = TempDir::new().unwrap();
        let agent_path = dir.path().join("analyst.toml");
        std::fs::write(
            &agent_path,
            r#"
role = "process-analyst"
model = "claude-sonnet-4.5"
tools = ["workspace_read_file", "workspace_search", "folder_structure"]
temperature = 0.5
folder_types = ["bpmn-simulator"]
autonomy = "supervised"
system_prompt = """
# Process Analyst

You analyze BPMN process models and suggest improvements.
"""
"#,
        )
        .unwrap();

        let agent = load_agent(&agent_path).unwrap();
        assert_eq!(agent.name, "analyst");
        assert_eq!(agent.role, "process-analyst");
        assert_eq!(agent.model, "claude-sonnet-4.5");
        assert_eq!(agent.temperature, 0.5);
        assert_eq!(agent.autonomy, "supervised");
        assert_eq!(agent.format, "toml");
        assert!(agent.active);
        assert!(agent.system_prompt.contains("# Process Analyst"));
    }

    #[test]
    fn test_discover_agents_all_formats() {
        let dir = TempDir::new().unwrap();

        std::fs::write(
            dir.path().join("writer.md"),
            "---\nrole: writer\ntools: [workspace_read_file]\n---\n# Writer\nYou write.",
        ).unwrap();
        std::fs::write(
            dir.path().join("reviewer.yaml"),
            "role: reviewer\ntools: [workspace_read_file]\nsystem_prompt: |\n  You review.",
        ).unwrap();
        std::fs::write(
            dir.path().join("planner.toml"),
            "role = \"planner\"\ntools = [\"workspace_read_file\"]\nsystem_prompt = \"You plan.\"",
        ).unwrap();

        let agents = discover_agents(dir.path()).unwrap();
        assert_eq!(agents.len(), 3);
        let formats: Vec<&str> = agents.iter().map(|a| a.format.as_str()).collect();
        assert!(formats.contains(&"md"));
        assert!(formats.contains(&"yaml"));
        assert!(formats.contains(&"toml"));
    }

    #[test]
    fn test_validation_invalid_tool() {
        let agent = AgentDefinition {
            tools: vec!["workspace_read_file".to_string(), "hack_the_planet".to_string()],
            system_prompt: "Do stuff.".to_string(),
            ..Default::default()
        };
        let errors = validate_agent(&agent);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "tools" && e.message.contains("hack_the_planet")));
    }

    #[test]
    fn test_validation_invalid_autonomy() {
        let agent = AgentDefinition {
            autonomy: "reckless".to_string(),
            system_prompt: "Do stuff.".to_string(),
            ..Default::default()
        };
        let errors = validate_agent(&agent);
        assert!(errors.iter().any(|e| e.field == "autonomy"));
    }

    #[test]
    fn test_validation_empty_prompt() {
        let agent = AgentDefinition {
            system_prompt: "   ".to_string(),
            ..Default::default()
        };
        let errors = validate_agent(&agent);
        assert!(errors.iter().any(|e| e.field == "system_prompt"));
    }

    #[test]
    fn test_validation_temperature_out_of_range() {
        let agent = AgentDefinition {
            temperature: 3.5,
            system_prompt: "Do stuff.".to_string(),
            ..Default::default()
        };
        let errors = validate_agent(&agent);
        assert!(errors.iter().any(|e| e.field == "temperature"));
    }

    #[test]
    fn test_load_invalid_agent_marked_inactive() {
        let dir = TempDir::new().unwrap();
        let agent_path = dir.path().join("broken.yaml");
        std::fs::write(
            &agent_path,
            r#"
role: broken-agent
model: claude-sonnet-4.5
tools: [nonexistent_tool]
autonomy: yolo
system_prompt: ""
"#,
        )
        .unwrap();

        let agent = load_agent(&agent_path).unwrap();
        assert!(!agent.active);
        assert!(agent.validation_errors.len() >= 2); // bad tool + bad autonomy + empty prompt
    }

    #[test]
    fn test_discover_agents_dual_format() {
        let dir = TempDir::new().unwrap();

        std::fs::write(
            dir.path().join("alpha.md"),
            "---\nrole: alpha\ntools: [workspace_read_file]\n---\n# Alpha Agent\nYou are alpha.",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("beta.yaml"),
            "role: beta\ntools: [workspace_read_file]\nsystem_prompt: |\n  # Beta Agent\n  You are beta.",
        )
        .unwrap();
        // Non-agent file should be ignored
        std::fs::write(dir.path().join("readme.txt"), "not an agent").unwrap();

        let agents = discover_agents(dir.path()).unwrap();
        assert_eq!(agents.len(), 2);
        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert_eq!(
            agents.iter().find(|a| a.name == "alpha").unwrap().format,
            "md"
        );
        assert_eq!(
            agents.iter().find(|a| a.name == "beta").unwrap().format,
            "yaml"
        );
    }

    #[test]
    fn test_export_for_zeroclaw_skips_inactive() {
        let agents = vec![
            AgentDefinition {
                name: "good".to_string(),
                active: true,
                system_prompt: "valid".to_string(),
                ..Default::default()
            },
            AgentDefinition {
                name: "bad".to_string(),
                active: false,
                system_prompt: "invalid".to_string(),
                ..Default::default()
            },
        ];

        let result = export_for_zeroclaw(&agents).unwrap();
        let exported = result["agents"].as_array().unwrap();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0]["name"], "good");
    }

    #[test]
    fn test_active_agents_filter() {
        let agents = vec![
            AgentDefinition {
                name: "active1".to_string(),
                active: true,
                ..Default::default()
            },
            AgentDefinition {
                name: "inactive".to_string(),
                active: false,
                ..Default::default()
            },
            AgentDefinition {
                name: "active2".to_string(),
                active: true,
                ..Default::default()
            },
        ];

        let active = active_agents(&agents);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|a| a.active));
    }
}

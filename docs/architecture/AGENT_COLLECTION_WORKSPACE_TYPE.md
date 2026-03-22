# Agent Collection Workspace Type

## Overview

The `agent-collection` folder type stores AI agent definitions that can be used across the workspace. Agents are defined as individual files (`.md`, `.yaml`, `.yml`, or `.toml`) and are validated, indexed, and matched to folder types automatically.

## Folder Type Registration

**FolderType enum variant:** `AgentCollection`
**YAML identifier:** `agent-collection`
**Builtin definition:** `crates/workspace-manager/src/builtin_types/agent-collection.yaml`

```yaml
# workspace.yaml
folders:
  my-agents:
    type: agent-collection
```

## Processor Crate

**Location:** `crates/workspace-processors/agent-collection/`
**Package name:** `agent-collection-processor`

### Core Struct: `AgentDefinition`

```rust
pub struct AgentDefinition {
    pub name: String,              // derived from filename
    pub role: String,              // matches folder type agent_roles
    pub description: String,       // human-readable summary
    pub model: String,             // LLM model identifier
    pub tools: Vec<String>,        // allowed workspace tools
    pub temperature: f32,          // 0.0–2.0
    pub folder_types: Vec<String>, // compatible types (empty = all)
    pub autonomy: String,          // autonomous | supervised | manual
    pub max_iterations: u32,       // tool-use loop limit (default: 10)
    pub max_tokens: u32,           // output token limit (default: 4096)
    pub metadata: HashMap<String, serde_yaml::Value>,
    pub system_prompt: String,     // agent instructions
    pub format: String,            // "md", "yaml", or "toml"
    pub active: bool,              // false if validation failed
    pub validation_errors: Vec<ValidationError>,
}
```

### Supported Formats

| Format | Extension | System Prompt Source |
|---|---|---|
| Markdown | `.md` | Markdown body (below frontmatter) |
| YAML | `.yaml`, `.yml` | `system_prompt` field |
| TOML | `.toml` | `system_prompt` field |

All three formats support the same fields. The markdown format uses YAML frontmatter for metadata.

### Key Functions

| Function | Description |
|---|---|
| `load_agent(path)` | Parse a single agent file (any format) |
| `discover_agents(folder_path)` | Find all `.md/.yaml/.yml/.toml` files |
| `load_collection(folder_path, config)` | Load from config or auto-discover |
| `validate_agent(&mut def)` | Run validation checks, set `active` flag |
| `active_agents(agents)` | Filter to valid agents only |
| `export_for_zeroclaw(agents)` | ZeroClaw session config JSON |
| `export_for_claude_code(agents)` | Claude Code CLI JSON |
| `export_for_api(agents)` | Claude API messages JSON |

### Validation

Agents are validated on load. Invalid agents are marked `active: false`.

- **Tools:** Must be one of: `workspace_read_file`, `workspace_write_file`, `workspace_list_files`, `workspace_search`, `folder_structure`, `workspace_context`
- **Autonomy:** Must be: `autonomous`, `supervised`, `manual`
- **Temperature:** Must be between 0.0 and 2.0
- **System prompt:** Must not be empty

## Agent Viewer

Files opened inside `agent-collection` folders render with a custom viewer (`agent_viewer.html`) instead of the generic markdown/text preview.

**Detection:** `open_file_page` handler checks the parent folder's type via `WorkspaceConfig::load()`. If typed as `agent-collection`, loads the agent with `load_agent()` and renders `AgentViewerTemplate`.

**Viewer features:**
- Metadata card with role, autonomy, format, and active/inactive badges
- Properties grid (model, temperature, autonomy, max_iterations, max_tokens)
- Tools and folder types as badge lists
- Export panel with ZeroClaw / Claude Code / API tabs
- Validate button, source toggle, sidebar with sibling agents
- Rendered system prompt (markdown to HTML)

## Two-Way Agent Matching

1. Folder types declare expected roles via `agent_roles` in their YAML definition
2. Agents declare compatible folder types via `folder_types`
3. The `/folders/agents?path=` endpoint returns agents where both sides match
4. Agents with empty `folder_types` match all folder types
5. Only `active` agents are returned

## Files

```
crates/workspace-processors/agent-collection/
  Cargo.toml                     # deps: serde, serde_yaml, serde_json, toml, anyhow
  src/lib.rs                     # parsing, validation, discovery, export (16 tests)

crates/workspace-manager/
  src/builtin_types/agent-collection.yaml   # folder type definition
  src/lib.rs                     # AgentViewerTemplate, handler detection logic
  templates/workspaces/agent_viewer.html    # custom viewer template
```

# AI Agent Framework

Specialized AI agents for different folder types that can autonomously create and edit workspace content, with ZeroClaw integration planned as the execution engine.

## Architecture

```
video-server-rs (domain knowledge)     ZeroClaw (execution engine)
  - folder types + agent roles    -->    agent loop
  - agent definitions (.md/.yaml/.toml) -->  tool dispatch
  - workspace file tools          <--    tool callbacks
  - approval flow (WebSocket)     <--    supervised writes
```

**video-server-rs** provides: folder type definitions, agent discovery, validation, file context gathering, tool execution.
**ZeroClaw** (future sidecar) provides: agentic loop, LLM orchestration, sandboxing, multi-agent coordination.

A built-in fallback mode (direct LLM chat with agent prompts) works without ZeroClaw.

## Components

### 1. Agent Roles in Folder Types

Each folder type YAML defines `agent_roles` — the roles that make sense for that content type:

```yaml
# builtin_types/course.yaml
agent_roles:
  - role: content-writer
    description: Creates and edits course modules and lessons
    default_actions:
      - Write new lesson
      - Edit lesson content
      - Generate quiz questions
  - role: course-planner
    description: Plans course structure, outlines modules and learning paths
    default_actions:
      - Plan module structure
      - Generate course outline
```

All 10 builtin types have agent roles defined. The `default_actions` list populates quick-action buttons in the UI.

**Struct:** `AgentRole` in `crates/workspace-manager/src/folder_type_registry.rs`

**Roles by type:**

| Folder Type | Agent Roles |
|---|---|
| static-site | content-writer, seo-optimizer |
| course | content-writer, course-planner |
| presentation | content-writer, slide-designer |
| documentation | content-writer, doc-reviewer |
| bpmn-simulator | process-modeler |
| data-pipeline | pipeline-engineer |
| js-tool | tool-builder |
| agent-collection | agent-designer |
| media-server | media-curator |
| yhm-site-data | content-writer |

### 2. Agent Definitions (Multi-Format)

Agents can be defined in three formats, stored in `agent-collection` typed folders:

#### Markdown (.md)

YAML frontmatter defines the agent metadata; the markdown body becomes the system prompt.

```markdown
---
role: content-writer
description: Creates and edits structured content for websites and courses
model: claude-sonnet-4.5
tools:
  - workspace_read_file
  - workspace_write_file
  - workspace_list_files
temperature: 0.7
folder_types:
  - static-site
  - course
autonomy: supervised
max_iterations: 15
max_tokens: 8192
---

You create and edit content following the folder's structure conventions.
Always read existing files first to understand the style and format.
```

#### YAML (.yaml / .yml)

All fields in a single YAML document. The `system_prompt` field replaces the markdown body.

```yaml
role: content-writer
description: Creates and edits structured content for websites and courses
model: claude-sonnet-4.5
tools:
  - workspace_read_file
  - workspace_write_file
  - workspace_list_files
temperature: 0.7
folder_types:
  - static-site
  - course
autonomy: supervised
max_iterations: 15
max_tokens: 8192
system_prompt: |
  You create and edit content following the folder's structure conventions.
  Always read existing files first to understand the style and format.
```

#### TOML (.toml)

Same structure as YAML but in TOML syntax.

```toml
role = "content-writer"
description = "Creates and edits structured content for websites and courses"
model = "claude-sonnet-4.5"
tools = ["workspace_read_file", "workspace_write_file", "workspace_list_files"]
temperature = 0.7
folder_types = ["static-site", "course"]
autonomy = "supervised"
max_iterations = 15
max_tokens = 8192
system_prompt = """
You create and edit content following the folder's structure conventions.
Always read existing files first to understand the style and format.
"""
```

#### Field Reference

| Field | Type | Default | Description |
|---|---|---|---|
| `role` | string | `"unknown"` | Matches `agent_roles` in folder type definitions |
| `description` | string | `""` | Human-readable summary of what the agent does |
| `model` | string | `"claude-sonnet-4.5"` | LLM model identifier |
| `tools` | string[] | `[]` | Allowed tool names (validated against known tools) |
| `temperature` | float | `1.0` | LLM temperature (0.0–2.0) |
| `folder_types` | string[] | `[]` | Compatible folder types (empty = all types) |
| `autonomy` | string | `"supervised"` | `autonomous`, `supervised`, or `manual` |
| `max_iterations` | u32 | `10` | Max tool-use iterations before the agent stops |
| `max_tokens` | u32 | `4096` | Max output tokens per LLM call |
| `system_prompt` | string | — | Agent instructions (in .md files, the body is the prompt) |

Extra/unknown fields in YAML/TOML are preserved in a `metadata` map and passed through in exports.

**Agent name:** Derived from the filename (without extension), not from a field.

### 3. Validation

Agent definitions are validated on load. Invalid agents are marked `active: false` and excluded from the agent panel and exports.

**Validation rules:**

| Check | Error |
|---|---|
| Unknown tool name | `"Unknown tool: {name}. Valid tools: ..."` |
| Invalid autonomy | `"Invalid autonomy: {val}. Must be: autonomous, supervised, manual"` |
| Temperature < 0 or > 2 | `"Temperature must be between 0.0 and 2.0"` |
| Empty system prompt | `"System prompt is empty"` |

**Valid tool names:** `workspace_read_file`, `workspace_write_file`, `workspace_list_files`, `workspace_search`, `folder_structure`, `workspace_context`

**Valid autonomy levels:** `autonomous`, `supervised`, `manual`

**Key functions:**
- `validate_agent(&mut def)` — runs all checks, populates `validation_errors`, sets `active`
- `active_agents(agents)` — filters a collection to only valid agents

Validation errors are displayed in the agent viewer UI as a red alert banner.

### 4. Agent Viewer

When opening an agent file (`.md`, `.yaml`, `.yml`, `.toml`) inside an `agent-collection` folder, the workspace browser renders a custom agent viewer instead of the generic markdown/text preview.

**Template:** `crates/workspace-manager/templates/workspaces/agent_viewer.html`

**Features:**
- Header with agent name and description
- Validation error alert (red banner) when the agent is inactive
- Metadata card with color-coded badges: role, autonomy (green/yellow/blue), format, active/inactive
- Properties grid: Model, Temperature, Autonomy, Max Iterations, Max Tokens
- Tools list (monospace badges)
- Folder types list (accent badges)
- Export panel with tabs: ZeroClaw, Claude Code, API — each showing the JSON export config
- Validate button (reloads the page to re-run validation)
- Source toggle (shows the raw file content)
- Sidebar listing sibling agent files in the same folder
- Rendered system prompt (markdown → HTML)

**Detection logic:** The `open_file_page` handler in `crates/workspace-manager/src/lib.rs` checks if the file's parent folder is typed as `agent-collection` via `WorkspaceConfig::load()`. If so, it loads the agent definition with `agent_collection_processor::load_agent()` and renders `AgentViewerTemplate` instead of the default viewer.

**Struct:** `AgentViewerTemplate` in `crates/workspace-manager/src/lib.rs`

### 5. Agent Tools

Domain-specific tools that agents use to interact with workspace files.

**Crate:** `crates/agent-tools/`

| Tool | Description |
|---|---|
| `workspace_read_file` | Read a file from the workspace |
| `workspace_write_file` | Write/create a file (with path validation) |
| `workspace_list_files` | List folder contents |
| `workspace_search` | Search file contents (case-insensitive grep) |
| `folder_structure` | Get folder type info and structure |
| `workspace_context` | Get full workspace context |

All tools include JSON Schema definitions for LLM function calling and enforce path traversal protection via `safe_resolve()`.

**Dispatch:** `agent_tools::dispatch_tool(workspace_root, tool_name, params)` routes calls by name.

### 6. Export Formats

Agent definitions can be exported in three formats for integration with external systems. Only `active` (valid) agents are included in exports.

#### ZeroClaw Format

```json
{
  "agents": [{
    "name": "content-writer",
    "role": "content-writer",
    "description": "Creates and edits structured content",
    "model": "claude-sonnet-4.5",
    "system_prompt": "...",
    "tools": ["workspace_read_file", "workspace_write_file"],
    "temperature": 0.7,
    "autonomy": "supervised",
    "max_iterations": 15,
    "max_tokens": 8192,
    "folder_types": ["static-site", "course"]
  }]
}
```

#### Claude Code Format

```json
{
  "agents": [{
    "name": "content-writer",
    "model": "claude-sonnet-4.5",
    "system_prompt": "...",
    "tools": ["workspace_read_file"],
    "temperature": 0.7,
    "max_tokens": 8192
  }]
}
```

#### API Format (Claude API messages)

```json
{
  "agents": [{
    "name": "content-writer",
    "model": "claude-sonnet-4.5",
    "system": "...",
    "tools": [{"name": "workspace_read_file", "type": "custom"}],
    "temperature": 0.7,
    "max_tokens": 8192
  }]
}
```

### 7. API Endpoints

All under `/api/workspaces/{workspace_id}/`:

| Method | Path | Description |
|---|---|---|
| GET | `/agents` | List all agents from agent-collection folders |
| GET | `/folders/agents?path=` | Agents compatible with a folder's type |
| GET | `/folders/ai-context?path=` | Folder context for agent system prompts |
| POST | `/agent/tool` | Execute a tool call (for ZeroClaw callbacks) |
| GET | `/agent/tools` | List available tool definitions |
| POST | `/agents/export` | Export agents + tools in ZeroClaw format |

**Agent matching** (`/folders/agents`): Two-way compatibility check:
1. The folder type declares which roles it expects (`agent_roles`)
2. Each agent declares which folder types it supports (`folder_types`)
3. An agent with empty `folder_types` is compatible with all types
4. Only `active` (valid) agents are returned

**AI context** (`/folders/ai-context`): Returns folder type info, metadata, context files (50KB/file, 100KB total), and `ai-instructions.md` content if present.

### 8. UI: Agent Panel (File Browser)

A slide-out drawer accessible via the "Agents" button in the browser toolbar.

**File:** `static/js/panels/agent-panel.js`

**Features:**
- Lists agents compatible with the current folder type
- Shows expected roles from folder type definition (green dot = matched agent exists)
- Agent selection opens a chat interface
- Quick action buttons from agent role's `default_actions`
- Folder context automatically fetched and included in system prompt
- Approval prompt placeholder for future ZeroClaw supervised writes

**Context:** `window.browserContext` exposes `workspaceId`, `currentPath`, `folderTypeId`.

### 9. UI: Agent-Aware AI Panel (Editor)

The existing AI panel (`static/js/panels/ai-panel.js`) now supports agent selection.

**Additions:**
- Agent selector dropdown (shown when agents are available for the folder)
- Quick actions switch to the agent's role-specific actions
- System prompt uses the agent's markdown body instead of generic prompt
- Model auto-switches to the agent's configured model

## ZeroClaw Integration

When ZeroClaw is available as a sidecar:

1. **Agent activation:** User selects agent in UI -> server loads definition, merges with folder context -> sends to ZeroClaw as session config
2. **Tool execution:** ZeroClaw calls back to `/agent/tool` endpoint for file operations
3. **Approval flow:** `supervised` autonomy pauses on writes -> approval request pushed via WebSocket -> user approves/rejects in UI -> response sent back
4. **Export:** `/agents/export` provides the full config (agents + tools) in ZeroClaw format

**Compatibility fields:** `max_iterations` and `max_tokens` map directly to ZeroClaw's `AgentConfig`. The `autonomy` levels (`autonomous`, `supervised`, `manual`) match ZeroClaw's approval flow modes. The `tools` list maps to ZeroClaw's `ToolSpec` pattern.

Until ZeroClaw is integrated, the built-in chat mode (direct LLM streaming via `/api/llm/chat`) provides agent functionality using the agent's system prompt and folder context.

## Files

```
crates/workspace-manager/src/
  folder_type_registry.rs        # AgentRole struct, FolderTypeDefinition.agent_roles
  builtin_types/*.yaml           # 10 types with agent_roles
  lib.rs                         # API handlers + AgentViewerTemplate

crates/workspace-processors/agent-collection/src/
  lib.rs                         # Agent loading (.md/.yaml/.toml), validation, export

crates/agent-tools/src/
  lib.rs                         # Tool definitions, execution, dispatch

static/js/panels/
  agent-panel.js                 # Browser agent drawer (Alpine.js)
  ai-panel.js                    # Editor AI panel (agent-aware)

crates/workspace-manager/templates/workspaces/
  browser.html                   # Agent button + drawer in toolbar
  agent_viewer.html              # Custom agent definition viewer
```

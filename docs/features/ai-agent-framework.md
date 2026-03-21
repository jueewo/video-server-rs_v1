# AI Agent Framework

Specialized AI agents for different folder types that can autonomously create and edit workspace content, with ZeroClaw integration planned as the execution engine.

## Architecture

```
video-server-rs (domain knowledge)     ZeroClaw (execution engine)
  - folder types + agent roles    -->    agent loop
  - agent definitions (.md)       -->    tool dispatch
  - workspace file tools          <--    tool callbacks
  - approval flow (WebSocket)     <--    supervised writes
```

**video-server-rs** provides: folder type definitions, agent discovery, file context gathering, tool execution.
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

### 2. Agent Definitions (Markdown Files)

Agents are defined as markdown files with YAML frontmatter, stored in `agent-collection` typed folders:

```markdown
---
role: content-writer
model: claude-sonnet-4.5
tools: [read_file, write_file, list_files]
temperature: 0.7
folder_types: [static-site, course]
autonomy: supervised
---

# Content Writer Agent

You create and edit content following the folder's structure conventions.
Always read existing files first to understand the style and format.
```

**Key fields:**
- `role` — matches `agent_roles` in folder type definitions
- `model` — LLM model to use (overrides provider default)
- `tools` — tools the agent can use
- `folder_types` — which folder types this agent is compatible with (empty = all)
- `autonomy` — `supervised` (approval for writes), `autonomous`, or `manual`

The markdown body becomes the agent's system prompt.

**Crate:** `crates/workspace-processors/agent-collection/`

**Key functions:**
- `load_agent(path)` — parse a single `.md` agent file
- `discover_agents(folder_path)` — auto-discover all `.md` files in a folder
- `load_collection(folder_path, config)` — load from explicit config or auto-discover
- `export_for_zeroclaw(agents)` — convert to ZeroClaw session config
- `export_for_claude_code(agents)` — JSON format for Claude Code CLI
- `export_for_api(agents)` — Claude API request format

### 3. Agent Tools

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

### 4. API Endpoints

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

**AI context** (`/folders/ai-context`): Returns folder type info, metadata, context files (50KB/file, 100KB total), and `ai-instructions.md` content if present.

### 5. UI: Agent Panel (File Browser)

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

### 6. UI: Agent-Aware AI Panel (Editor)

The existing AI panel (`static/js/panels/ai-panel.js`) now supports agent selection.

**Additions:**
- Agent selector dropdown (shown when agents are available for the folder)
- Quick actions switch to the agent's role-specific actions
- System prompt uses the agent's markdown body instead of generic prompt
- Model auto-switches to the agent's configured model

## ZeroClaw Integration (Phase 2 — Future)

When ZeroClaw is available as a sidecar:

1. **Agent activation:** User selects agent in UI -> server loads definition, merges with folder context -> sends to ZeroClaw as session config
2. **Tool execution:** ZeroClaw calls back to `/agent/tool` endpoint for file operations
3. **Approval flow:** `supervised` autonomy pauses on writes -> approval request pushed via WebSocket -> user approves/rejects in UI -> response sent back
4. **Export:** `/agents/export` provides the full config (agents + tools) in ZeroClaw format

Until ZeroClaw is integrated, the built-in chat mode (direct LLM streaming via `/api/llm/chat`) provides agent functionality using the agent's system prompt and folder context.

## Files

```
crates/workspace-manager/src/
  folder_type_registry.rs        # AgentRole struct, FolderTypeDefinition.agent_roles
  builtin_types/*.yaml           # 10 types with agent_roles
  lib.rs                         # 6 new API handlers

crates/workspace-processors/agent-collection/src/
  lib.rs                         # Agent loading, discovery, export

crates/agent-tools/src/
  lib.rs                         # Tool definitions, execution, dispatch

static/js/panels/
  agent-panel.js                 # Browser agent drawer (Alpine.js)
  ai-panel.js                    # Editor AI panel (agent-aware)

crates/workspace-manager/templates/workspaces/
  browser.html                   # Agent button + drawer in toolbar
```

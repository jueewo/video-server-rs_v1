# 05 — Extensibility

## The Accidental Plugin System

Last roast's conclusion was: "The platform is modular internally but has no external extension surface." That's still partly true, but the agent framework has changed the equation in an interesting way.

### Extension Points — March 22 Update

| Extension Point | Type | External? | Status |
|----------------|------|-----------|--------|
| Folder types (YAML) | Configuration | Semi | Builtin only; custom types loaded from workspace config |
| FolderTypeRenderer (trait) | Code | No | Requires Rust crate + recompile |
| Agent definitions (.md) | Content | **Yes** | Users create agents as markdown files |
| Agent tools (dispatch) | Code | No | New tools require Rust code |
| REST API | HTTP | **Yes** | Growing surface, no OpenAPI spec |
| MCP server | Protocol | **Yes** | Scaffold only (5 TODO phases) |
| WebDAV | Protocol | **Yes** | Working for file access |
| LLM providers | Configuration | **Yes** | BYOK (Anthropic, OpenAI, Ollama, any OpenAI-compatible) |
| Access codes | Sharing | **Yes** | Working, but no UI for quick creation |
| Site generator | Build pipeline | No | Astro-based, tightly coupled |

### The Agent Framework Is an Extension System

Here's the non-obvious insight: **agent definitions are the first user-created extension point that actually works.**

Consider what an agent definition can do:
- Define a new behavior pattern (system prompt)
- Specify which folder types it applies to (folder_types field)
- Choose its own LLM model and temperature
- Declare its autonomy level
- Be shared between workspaces (copy the .md file)
- Be versioned (it's a file in a git-friendly workspace)
- Be included in workspace handoffs (it lives alongside content)

This is, functionally, a plugin. It's a user-defined extension that modifies platform behavior without code changes or recompilation. It just happens to be expressed as a markdown file instead of a JavaScript module.

### What's Missing to Make It a Real Plugin System

#### 1. Custom Tool Definitions

Currently, the 6 tools (`workspace_read_file`, `workspace_write_file`, etc.) are hardcoded in Rust. An agent can only use these tools, regardless of what its `tools` field says.

**To make it extensible:** Allow tool definitions as part of the agent or workspace configuration. A custom tool could be:
- A shell command template (risky but powerful)
- An HTTP endpoint call (safer, enables integration)
- A JavaScript function (most flexible, requires a JS runtime)

**Example:** A "publish-to-social" tool that posts a summary to a webhook. Or a "generate-image" tool that calls a DALL-E endpoint. These shouldn't require Rust code.

#### 2. Event Hooks

No event system exists. When a file is uploaded, transcoded, or shared, nothing happens beyond the immediate operation. Events would enable:
- Agent triggers: "When a new video is uploaded to this folder, run the content-writer agent to generate a description"
- Notifications: "Email the client when new content is added"
- Integrations: "Post to Slack when a workspace is shared"

**Implementation path:** A `hooks` table (event_type, workspace_id, action_type, action_config). Actions could be: run agent, call webhook, send email.

#### 3. Folder Type Definitions Without Recompile

Custom folder types are loaded from workspace config, but `FolderTypeRenderer` implementations require Rust. This means:
- You can define a new folder type's metadata (name, description, key files, agent roles) in YAML
- But you can't define how it renders or behaves without writing Rust

**To make it extensible:** A "generic" renderer that renders based on the YAML definition:
- Key files → shown prominently
- Metadata fields → shown as a form
- Agent roles → shown in agent panel
- Rendering → a customizable HTML template (Liquid, Tera, or even the file browser's default view)

This way, new folder types are *fully* YAML-defined, no Rust needed.

#### 4. Workspace Templates

Currently, creating a new workspace starts empty. There's no way to:
- Start from a template ("Consulting Engagement", "Training Course", "Client Portal")
- Clone an existing workspace's structure (without content)
- Share workspace blueprints between installations

**Implementation:** A `templates/` directory or a "template" flag on workspaces. When creating a new workspace, offer a dropdown: "Empty", "Consulting Engagement", "Training Course", "Custom."

### API Surface Assessment

The REST API has grown significantly:

**Media API** (stable, well-used):
- Upload, list, search, CRUD, serve
- HLS transcoding with WebSocket progress
- Thumbnail generation

**Workspace API** (growing):
- Workspace CRUD
- File browser, file editor
- Folder type management
- **New:** Agent discovery, AI context, tool execution, agent export

**Auth API** (stable):
- OIDC flow, session management
- Access codes, groups, API keys

**LLM API** (new):
- Provider listing, chat streaming

**What's missing:**
- **No OpenAPI spec** — the API is undocumented except in CLAUDE.md
- **No versioning** — all endpoints are unversioned (`/api/workspaces/...`). If you change a response shape, existing integrations break.
- **No rate limit headers** — clients can't see their remaining quota
- **No pagination** — list endpoints return everything. Fine for 50 items; problematic for 5,000.
- **No webhook registration** — no way for external systems to subscribe to events

### MCP Server: The Other Extension Path

The `media-mcp` crate is a scaffold with 5 TODO phases. MCP (Model Context Protocol) is gaining traction as the standard for connecting AI models to tools. A working MCP server would:
- Let Claude Desktop interact with AppKask workspaces natively
- Enable any MCP-compatible client to use AppKask's workspace tools
- Provide a standard interface that doesn't require custom API integration

**Priority assessment:** High value, moderate effort. The agent-tools crate already defines the exact tools that an MCP server would expose. The work is primarily protocol wrapping.

### ZeroClaw: The Execution Extension

ZeroClaw integration (when it arrives) would add:
- **Agentic loop** — multi-turn tool calling without browser-side orchestration
- **Multi-agent coordination** — agents collaborating on a task
- **Sandboxing** — agent operations contained and reversible
- **Session management** — long-running agent tasks that survive page refreshes

The current architecture (domain layer in AppKask, execution in ZeroClaw) is the right separation. AppKask provides the "what" (tools, context, agents); ZeroClaw provides the "how" (loop, safety, coordination).

### Extension Priority Ranking

| Extension | Effort | Value | Priority |
|-----------|--------|-------|----------|
| Agent loop (built-in) | 3-5 days | Enables core agent use case | 1 |
| Custom tool definitions | 1-2 weeks | Agent ecosystem growth | 2 |
| MCP server completion | 1 week | AI ecosystem integration | 3 |
| Workspace templates | 3-5 days | Onboarding, repeatability | 4 |
| Event hooks | 1-2 weeks | Automation, integration | 5 |
| Generic folder type renderer | 1-2 weeks | Community folder types | 6 |
| OpenAPI spec | 1 week | External integration enabler | 7 |
| ZeroClaw integration | 2-4 weeks | Advanced agent capabilities | 8 |

### Verdict

The agent framework has accidentally created AppKask's first real extension surface. Agent definitions (markdown + YAML) are a user-friendly, version-control-friendly, shareable plugin format. The next step isn't building a traditional plugin system (JavaScript modules, package registry, etc.) — it's deepening the agent-as-extension pattern:

1. Make agents functional (tool execution loop)
2. Make tools extensible (custom tool definitions)
3. Make agents shareable (templates, marketplace)
4. Make events hookable (trigger agents on file changes)

The traditional "npm install a plugin" model can come later. The markdown-file-as-plugin model is unique, fits the platform's identity, and is 80% built.

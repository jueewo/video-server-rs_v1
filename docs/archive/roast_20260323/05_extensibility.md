# 05 — Extensibility

## The Extension Surface Is Growing

Since March 22, three significant changes affect extensibility:

1. **DB trait abstraction** — The repository pattern enables swapping storage backends without touching business logic
2. **Agent registry** — A global workforce adds a new discovery and distribution dimension
3. **Workspace processors** — A framework for folder-type-specific processing pipelines

### Extension Points — March 23 Update

| Extension Point | Type | External? | Status | Change |
|----------------|------|-----------|--------|--------|
| Folder types (YAML) | Configuration | Semi | Working | Unchanged |
| FolderTypeRenderer (trait) | Code | No | Working | Unchanged |
| Workspace processors (trait) | Code | No | **New scaffolds** | +3 processor crates |
| Agent definitions (.md) | Content | **Yes** | Working | Unchanged |
| Agent registry (global) | Content + API | **Yes** | **New** | Global agent workforce |
| Agent tools (dispatch) | Code | No | Working | Unchanged |
| DB repositories (traits) | Code | No | **New** | 13 trait modules |
| REST API | HTTP | **Yes** | Growing | More endpoints |
| MCP server | Protocol | **Yes** | Scaffold only | Unchanged |
| WebDAV | Protocol | **Yes** | **Broken** | Build failure |
| LLM providers | Configuration | **Yes** | Working | Unchanged |
| Access codes | Sharing | **Yes** | Working | Unchanged |
| Federation | Protocol | **Yes** | **Improved** | Tenant-scoped, backoff |
| Site generator | Build pipeline | Semi | Working | Unchanged |

## The DB Trait Layer: The Silent Extension Enabler

The most impactful extensibility change this cycle isn't flashy — it's the repository trait abstraction.

**What it enables:**
- **PostgreSQL support** without changing any business logic crate
- **Test doubles** — mock repositories for unit testing without a database
- **Read replicas** — a repository impl that routes reads to a replica and writes to primary
- **Caching layer** — a repository wrapper that caches frequent queries
- **Audit wrapper** — a repository decorator that logs every data access

The 13 domain traits (`MediaRepository`, `AgentRepository`, `FederationRepository`, etc.) are the most compositionally powerful extension point in the codebase. They're not user-facing, but they're architecturally critical.

**Current gap:** No example of a non-SQLite implementation exists. Adding a `db-mock` crate with in-memory implementations would:
1. Prove the abstraction works
2. Enable fast unit tests
3. Serve as a template for future implementations

## Agent Registry: From Discovery to Distribution

The agent registry adds a new dimension to the agent extension story:

**March 22 model:**
```
Workspace → agent-collection folder → .md files → discovered agents
```

**March 23 model:**
```
Global registry → all available agents
    ↓ (filtered by)
Workspace → folder types → compatible agents
    ↓ (augmented by)
agent-collection folder → workspace-specific agents
```

This hierarchy enables:
- **Pre-installed agents** — ship AppKask with default agents for common folder types
- **Agent sharing** — export agents from one workspace, import to the registry
- **Agent marketplace** (future) — download agents from a community registry
- **Agent versioning** — registry could track agent versions and updates

**Current gap:** The relationship between registry agents and workspace agents is unclear. Define:
- Can a workspace-local agent override a registry agent with the same role?
- Are registry agents available in all workspaces by default?
- Can a user "install" a registry agent into a specific workspace?
- Can a user "publish" a workspace agent to the registry?

## Workspace Processors: The Folder Type Pipeline

The new `workspace-processors/` directory introduces a processing framework for folder types:

- `bpmn-simulator-processor` — BPMN simulation processing
- `static-site-processor` — Static site building
- More to come (presumably)

**Assessment:** This is the right pattern — each folder type can have a processor that handles build/transform operations. It parallels the renderer pattern (how a folder type displays) with a processor pattern (how a folder type transforms content).

**Current gap:** The processors are mostly TODOs. The static-site processor has 6 TODOs for framework builds (Astro, Next.js, etc.). The bpmn-simulator has 3 TODOs. These are scaffolds, not implementations.

**Risk:** Too many extension patterns competing for attention. The codebase now has:
1. Folder type renderers (display)
2. Folder type processors (transform)
3. Agent definitions (AI behavior)
4. Agent tools (AI capabilities)
5. Agent registry (AI discovery)
6. DB repositories (data access)
7. Federation (cross-instance)

Each is individually well-designed, but the combined cognitive load for a contributor is high. Consider a "contributor guide" that maps: "I want to add X" → "you need to touch Y."

## API Surface Assessment (Updated)

The API has grown with new agent registry and tenant management endpoints:

**Media API** (stable):
- Upload, list, search, CRUD, serve
- HLS transcoding with WebSocket progress
- Multi-tenant scoped (tenant_id)

**Workspace API** (growing):
- CRUD, file browser, file editor, file ops
- Folder type management
- Agent discovery, AI context, tool execution
- Site generation and publishing
- Tenant administration

**Agent API** (new):
- Registry CRUD
- Detail pages
- Import/export

**Federation API** (improved):
- Peer management with failure tracking
- Tenant-scoped catalog sharing
- Proxy with caching

**Auth API** (stable):
- OIDC, sessions, access codes, groups, API keys

**LLM API** (stable):
- Provider listing, chat streaming

**Still missing (unchanged):**
- No OpenAPI spec
- No versioning
- No rate limit headers
- No pagination on list endpoints
- No webhook registration

The lack of pagination is becoming more pressing with multi-tenant support. A single instance serving multiple tenants could easily accumulate thousands of media items. List endpoints that return everything will degrade.

## MCP Server: Still a Scaffold

No progress since March 22. `media-mcp/src/main.rs` is 60 lines with 5 TODO phases.

**Updated priority:** With the DB trait abstraction complete, the MCP server could use repository traits directly instead of HTTP. This would be cleaner and faster. But it's still a scaffold.

## WebDAV: Broken

The WebDAV crate doesn't compile after the DB trait migration. This is a functional extension point that went from "working" to "broken." Fix it or explicitly deprecate it.

## Extension Ecosystem Maturity

| Layer | March 22 | March 23 | Status |
|-------|----------|----------|--------|
| Data access | sqlx everywhere | Trait-based, swappable | Mature |
| AI agents | Per-workspace discovery | + Global registry | Growing |
| Processing | None | Processor framework (scaffolds) | Early |
| Federation | Basic | Tenant-scoped, resilient | Mature |
| External API | Growing, undocumented | Still growing, still undocumented | Needs spec |
| Protocols (MCP, WebDAV) | Scaffolds | One broken, one still scaffold | Regression |

## Verdict

The extensibility story improved significantly at the infrastructure level (DB traits) and agent level (registry). The platform now has 7 distinct extension patterns, which is both powerful and potentially overwhelming. The immediate priorities are:
1. Fix WebDAV (don't ship broken extension points)
2. Document the agent registry/workspace relationship
3. Add pagination to API list endpoints
4. Create a contributor guide mapping use cases to extension points

The "agent definitions as plugins" story from March 22 is stronger now with the global registry. The path from "agents in my workspace" to "agents from a marketplace" is clearer. But the execution gap (no agentic loop, processors are TODOs, MCP is a scaffold) means the extensibility story is still more promise than reality.

# 04 — Improvements

Ranked by effort and impact. Items carried over from March 16 are marked with (C).

---

## Tier 1 — This Week (High Impact, Low Effort)

### 1. Add CI Pipeline (C)
**Effort:** 1-2 hours
**Impact:** Catches regressions, signals professionalism to contributors

You now have ~20 tests. They should run on every push. A minimal GitHub Actions workflow:
```yaml
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
```

No excuses. 15 minutes. Do it.

### 2. Split workspace-manager/src/lib.rs
**Effort:** 2-4 hours
**Impact:** Makes the largest crate maintainable

This file is 6,030 lines. Extract:
- `agent_handlers.rs` — 6 agent API handlers (~300 lines)
- `ai_context.rs` — context gathering logic (~150 lines)
- Keep workspace CRUD and file browser in lib.rs (for now)

Start with the agents module since it's the newest and has the fewest dependencies on the rest of the file.

### 3. Docker Image
**Effort:** 4-8 hours
**Impact:** Makes AppKask installable by non-Rust-developers

Currently, installing AppKask requires: Rust toolchain, FFmpeg, ffprobe, MediaMTX, Ghostscript, cwebp. That's 6 system dependencies. A `Dockerfile` with a multi-stage build (Rust builder → slim runtime with all deps) would:
- Let Maria deploy on her VPS
- Enable demo instances
- Reduce "works on my machine" issues

### 4. Demo Workspace on First Login
**Effort:** 4-6 hours
**Impact:** Transforms onboarding from "what do I do?" to "oh, I see"

On first login (no workspaces exist), auto-create a "Welcome to AppKask" workspace with:
- A `media-server` folder with a sample image and video
- A `course` folder with a sample lesson (markdown)
- An `agent-collection` folder with a "Content Writer" agent
- A README explaining the platform's concepts

This single change would address the biggest usability gap.

---

## Tier 2 — This Month (High Impact, Medium Effort)

### 5. Agent Agentic Loop (Tool Execution via Chat)
**Effort:** 3-5 days
**Impact:** Makes the agent framework actually useful

Currently, agents chat but can't execute tools. The framework is all dressed up with nowhere to go. Implement:
1. Detect tool-use responses from the LLM (function call JSON in the stream)
2. Execute via `agent_tools::dispatch_tool()`
3. Feed result back to the LLM as a tool response
4. Repeat until the agent produces a text response
5. For `supervised` autonomy: pause before write operations, show approval UI

This is the #1 feature that turns AppKask from "a chat interface with context" into "an AI-powered content platform."

### 6. Workspace Search
**Effort:** 2-3 days
**Impact:** Basic expectation for any content platform

You have `workspace_search` as an agent tool. Expose it as a user-facing feature:
- Search bar in workspace browser toolbar
- Results show matching files with context snippets
- Filter by folder, file type, date range

### 7. Move Vendored JS to Package Manager (C)
**Effort:** 2-4 days (with bun)
**Impact:** Reduces repo size, enables security updates

Use bun (user preference) to manage Alpine.js, DaisyUI, Reveal.js, etc. Output to `static/dist/`. Add `node_modules/` to `.gitignore`.

### 8. Breadcrumb Navigation
**Effort:** 1 day
**Impact:** Basic spatial orientation for nested folder structures

The workspace browser has no breadcrumbs. For a platform built around folder hierarchies, this is a significant orientation gap.

### 9. Agent Creation UI
**Effort:** 2-3 days
**Impact:** Makes agents accessible to non-developers

A "Create Agent" form in the agent panel:
- Name, role (dropdown from folder type's expected roles), model (from configured providers)
- System prompt (textarea with template)
- Folder types (checkboxes)
- Autonomy level (radio: supervised/autonomous/manual)
- Saves as `.md` file in the workspace's agent-collection folder

---

## Tier 3 — This Quarter (Medium Impact, Significant Effort)

### 10. One-Click Sharing (C)
**Effort:** 3-5 days
**Impact:** Fulfills the "share via access code" promise

Currently creating an access code requires navigating to a separate management page. It should be:
- Click "Share" button on any folder
- Modal: choose expiry, generate code, copy link
- Link format: `https://your-domain.com/s/{code}`

### 11. Unified Error Types (C)
**Effort:** 3-5 days
**Impact:** Consistent error responses across all endpoints

Define `AppError` enum with:
- `NotFound`, `Unauthorized`, `Forbidden`, `BadRequest`, `Internal`
- `IntoResponse` impl that returns structured JSON
- Replace all `(StatusCode, String)` returns

### 12. OpenAPI Specification
**Effort:** 1 week
**Impact:** Enables external integrations, SDK generation, API documentation

The API surface is growing (media, workspaces, agents, LLM, auth). Without a spec:
- No one can build integrations without reading source code
- The MCP server and CLI have to guess at endpoints
- No Swagger UI for exploring the API

Consider `utoipa` crate for derive-macro-based OpenAPI generation.

### 13. Audit Log UI
**Effort:** 3-5 days
**Impact:** Required for the "privacy-conscious companies" audience

If you're targeting regulated industries, audit visibility is table stakes. A page showing: who accessed what, when, from where. The data likely exists in logs; it needs a UI.

### 14. Background Job System (C)
**Effort:** 1-2 weeks
**Impact:** Proper handling of transcoding, site builds, agent tasks

Currently, background work is `tokio::spawn` with DashMap progress tracking. This works but:
- No persistence (server restart loses in-flight jobs)
- No retry on failure
- No job queue (all jobs run immediately, no concurrency limits)

Consider a lightweight job table in SQLite + a worker loop. Not a full job framework — just enough for reliability.

---

## Tier 4 — Strategic (High Effort, High Long-Term Value)

### 15. ZeroClaw Integration
**Effort:** 2-4 weeks
**Impact:** Transforms agents from chat assistants to autonomous workers

The agent framework's real potential is blocked on the execution engine. ZeroClaw provides:
- Agentic loop with tool calling
- Sandboxing
- Multi-agent coordination
- Supervised execution

However: if ZeroClaw integration proves complex, a built-in loop (Tier 2, #5) gives 80% of the value.

### 16. Plugin System (C)
**Effort:** 2-4 weeks
**Impact:** Enables community-built folder types, agents, and tools

Current extension model: write a Rust crate, implement `FolderTypeRenderer`, recompile. That's a developer tool, not a plugin system. Consider:
- JavaScript plugins (fastest, most accessible)
- Agent definitions as the first "plugin format" (already markdown-based!)
- Folder type definitions as the second (already YAML-based!)
- Tool definitions as the third

The agent framework has accidentally created the beginnings of a plugin system. Lean into it.

### 17. PostgreSQL Option
**Effort:** 2-3 weeks
**Impact:** Removes the SQLite ceiling for larger deployments

Keep SQLite as default. Add PostgreSQL as a configuration option for teams that need concurrent writes, replication, or existing database infrastructure. sqlx already supports both — the main work is query compatibility and migration management.

---

## Priority Matrix

```
                    HIGH IMPACT
                        |
         [1. CI]  [4. Demo WS]  [5. Agent Loop]
         [2. Split lib.rs]      [6. Search]
                        |
  LOW EFFORT ———————————+——————————————— HIGH EFFORT
                        |
         [3. Docker]    |       [15. ZeroClaw]
         [8. Breadcrumbs]       [16. Plugins]
                        |       [17. PostgreSQL]
                    LOW IMPACT
```

**The critical path:** CI (#1) → Split lib.rs (#2) → Docker (#3) → Demo workspace (#4) → Agent loop (#5). These five items, in order, would transform AppKask from a developer tool into a presentable product.

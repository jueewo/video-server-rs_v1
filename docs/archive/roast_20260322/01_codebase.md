# 01 — Codebase

## Metrics Snapshot

| Metric | March 16 | March 22 | Trend |
|--------|----------|----------|-------|
| Workspace crates | 34 | 37 | Growing |
| Rust source files | 139 | ~495 | Large jump (includes generated/template) |
| Handwritten Rust LOC | ~55,000 | ~63,000 | Steady growth |
| Askama templates | 331 | ~340 | Slight growth |
| Unit tests | 0 | ~20 | Finally some |
| Compiler warnings | Multiple | 0 | Clean |
| TODO/FIXME comments | Not tracked | 3 (all future-phase markers) | Low debt |
| Largest file (lib.rs) | 4,978 LOC | 6,030 LOC | workspace-manager grew |
| main.rs | 1,538 LOC | 1,684 LOC | Still growing |

## Architecture: What's Good

### 1. The Agent Framework Is Well-Designed

The three new crates (`agent-tools`, `agent-collection` rewrite, folder-type agent roles) follow the established crate-per-concern pattern. Specific wins:

- **Two-way compatibility matching** is elegant: folder types declare roles they expect, agents declare folder types they support, empty `folder_types` means "I work everywhere." This avoids the N*M explosion problem.
- **Agent definitions as markdown** is a genuinely good idea. Non-developers can read and edit agent prompts. The YAML frontmatter carries structured config. The body becomes the system prompt. It's the same pattern that made Jekyll/Hugo successful for content.
- **`safe_resolve()`** in agent-tools prevents path traversal on every tool call. This is the kind of security-by-default that matters when you're giving LLMs write access to a filesystem.
- **Export functions** (`export_for_zeroclaw`, `export_for_claude_code`, `export_for_api`) show forward-thinking. You built the domain layer independent of the execution engine. When ZeroClaw (or anything else) shows up, you plug it in without rewriting.

### 2. Test Coverage Finally Exists (Barely)

- `agent-tools`: 6 tests including read/write roundtrip, search, and path traversal blocking
- `agent-collection`: 7 tests including frontmatter parsing, discovery, and export
- `folder-type-registry`: Updated tests for agent roles, type counts, overwrite behavior

This is still laughably thin for 63K LOC, but it's not zero anymore. The tests that *do* exist cover security-critical paths (path traversal) and data parsing (agent frontmatter). That's the right priority.

### 3. Zero Compiler Warnings

Commit `30d7dec` cleaned up all warnings. This matters more than it sounds — warnings are broken windows. When there are 50 warnings, nobody notices a new one. When there are 0, every new one is visible.

### 4. Consistent Crate Patterns

The new crates follow the established patterns:
- State structs per domain
- `pub fn *_routes() -> Router` for route registration
- Explicit Askama rendering with `Html(template.render().map_err(...)?)`
- `serde` for serialization with `#[serde(default)]` for backwards compatibility

## Architecture: What's Concerning

### 1. The God File Grew

`workspace-manager/src/lib.rs` went from 4,978 to 6,030 lines. Six new API handlers were added inline. This file now handles:
- Workspace CRUD
- File browsing
- File editing
- Folder type management
- Agent discovery and matching
- Agent tool execution
- AI context gathering
- Agent export

That's at least 4 distinct responsibilities. The file should have been split *before* adding agent handlers, not after. Every new feature makes the eventual split harder.

**Recommendation:** Extract into submodules NOW:
- `workspace_crud.rs` — workspace create/read/update/delete
- `file_browser.rs` — browsing, editing, file operations (may already exist partially)
- `agent_handlers.rs` — all 6 new agent endpoints
- `ai_context.rs` — context gathering for agent prompts

### 2. main.rs Keeps Growing Too

1,684 lines. It wires up 10+ state structs, registers dozens of routes, handles production validation, OTLP setup, session management, and static file serving. Classic "everything connects here" problem.

**Recommendation:** Route registration should move to per-domain functions that take shared state and return a `Router`. main.rs should be <500 lines: create pool, create states, merge routers, start server.

### 3. Inline SQL Everywhere

No query modules or repository pattern. SQL strings are scattered across handler functions. This makes:
- Schema changes painful (grep for column names)
- Query reuse impossible
- Testing harder (can't mock the data layer)

Not a blocker, but as the schema grows, this will slow you down.

### 4. No CI Pipeline (Still)

This was item #2 in the last roast. Still not done. You now have tests that *could* run in CI. They don't. A GitHub Actions workflow with `cargo build && cargo test` would take 15 minutes to set up and would catch regressions on every push.

### 5. Vendored JS Still in Git (Still)

Hundreds of MB of JavaScript libraries committed to the repository. This bloats clones, makes diffs noisy, and prevents security updates via package managers.

## New Crate Assessment

### agent-tools (467 LOC)

**Quality:** High. Clean separation of concerns, each tool is a standalone function, dispatch is a simple match, `safe_resolve()` is correct and tested.

**Concern:** The `workspace_write_file` tool creates parent directories implicitly. This is convenient but means an agent can create arbitrarily nested directory structures. Consider a depth limit or explicit directory creation tool.

**Concern:** `workspace_search` does a synchronous recursive directory walk with `read_to_string` on every file. On a workspace with thousands of files (media server with video segments), this could be slow. Consider limiting search depth or file count.

### agent-collection (451 LOC)

**Quality:** Good. Frontmatter parsing is solid, export functions are well-structured.

**Concern:** The frontmatter parser splits on `---` and takes the first occurrence. If an agent's markdown body contains `---` (horizontal rules), it won't break parsing because the parser looks for the *opening* `---` at line 0. But document this assumption.

**Concern:** `discover_agents()` reads every `.md` file in a directory. If someone puts a README.md in an agent-collection folder, it'll try to parse it as an agent. Consider requiring a specific naming convention (e.g., `*.agent.md`) or checking for the `role` field before including.

### publications (802 LOC)

Not deeply reviewed, but the schema design (types, bundles, tags, access control) mirrors the established patterns. Looks clean.

## Code Smells

| Smell | Location | Severity |
|-------|----------|----------|
| God file | workspace-manager/src/lib.rs (6,030 LOC) | High |
| Growing main.rs | src/main.rs (1,684 LOC) | Medium |
| No repository pattern | SQL inline in handlers | Medium |
| No error type unification | Mix of anyhow, string errors, StatusCode | Medium |
| Unused standalone scaffolds | media-cli (8 TODO phases), media-mcp (5 TODO phases) | Low |
| No API documentation | No OpenAPI/Swagger spec | Medium |

## Verdict

The codebase is *better* than March 16. Zero warnings, real tests, well-designed new crates. But the structural problems (god files, no CI, vendored JS) weren't addressed. You're building a taller building on the same foundation. The agent framework is the strongest new addition — it's clean, well-separated, and forward-compatible. Channel that same discipline into splitting workspace-manager.

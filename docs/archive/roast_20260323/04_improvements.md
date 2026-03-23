# 04 — Improvements

Ranked by effort and impact. Items carried over from March 22 are marked with (C). Items from March 16 that have survived three roasts are marked with (C3).

---

## Tier 0 — Fix Today (Regressions)

### 1. Fix webdav Build Failure
**Effort:** 30 minutes
**Impact:** Unblocks `cargo build --workspace` for everyone

The webdav crate fails because `Pool<Sqlite>` doesn't implement `ApiKeyRepository`. This happened during the DB trait migration — webdav wasn't updated to use the new trait-based pattern. Either:
- Implement the missing trait
- Temporarily exclude webdav from the default workspace build (`default-members` in root Cargo.toml)

This is blocking. Fix it before anything else.

### 2. Fix 5 Doc-Test Compile Failures
**Effort:** 1 hour
**Impact:** Clean test output, accurate test count

All in `access-control`. The doc examples reference types or signatures that changed during the DB trait migration. Update the examples to match the current API.

### 3. Fix 2 Compiler Warnings
**Effort:** 5 minutes
**Impact:** Restore zero-warning status

Unused variables in `bpmn-simulator-processor`. Prefix with `_` or remove.

---

## Tier 1 — This Week (High Impact, Low Effort)

### 4. Add CI Pipeline (C3)
**Effort:** 1-2 hours
**Impact:** Catches the exact regressions introduced this cycle

This is the third consecutive roast recommending CI. You now have:
- 82 passing tests that *should* run on every push
- A build failure that CI *would* have caught
- Doc-test failures that CI *would* have caught
- Compiler warnings that CI *would* have caught

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

Every regression introduced this cycle would have been caught by these 3 commands. The cost of not having CI is no longer theoretical — it's the three regressions in this roast.

### 5. Split main.rs
**Effort:** 2-4 hours
**Impact:** Prevents the next god file

You demonstrated excellent modularization skills with the workspace-manager split. Apply the same pattern:

```
src/
  main.rs          → ~100 lines: create pool, build app, bind server
  config.rs        → AppConfig, DeploymentConfig, env parsing
  security.rs      → Production validation, OIDC setup, session config
  middleware.rs     → CORS, rate limiting, request-id, auth middleware
  routes.rs        → Route composition from all crates
  telemetry.rs     → OpenTelemetry setup
```

### 6. Docker Image (C)
**Effort:** 4-8 hours
**Impact:** Makes AppKask installable by anyone

Three roasts without Docker. The dependency list is now: Rust toolchain, FFmpeg, ffprobe, MediaMTX, Ghostscript, cwebp. That's 6 system dependencies that need to be manually installed and configured.

A multi-stage Dockerfile:
```dockerfile
# Stage 1: Build
FROM rust:1.77 AS builder
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get install -y ffmpeg ghostscript webp
COPY --from=builder /app/target/release/appkask /usr/local/bin/
# Bundle MediaMTX binary
```

`docker run -p 3000:3000 -v ./data:/data appkask/appkask` should be the entire install experience.

### 7. Clean Up Stale Worktrees
**Effort:** 5 minutes
**Impact:** Reduces noise in searches, saves disk space

```bash
rm -rf .claude/worktrees/*
```

Three stale worktrees with duplicated JS files.

---

## Tier 2 — This Month (High Impact, Medium Effort)

### 8. Demo Workspace on First Login (C)
**Effort:** 4-6 hours
**Impact:** Transforms onboarding

Still the single most impactful UX improvement. Auto-create on first login:
- A media folder with a sample image and video
- A course folder with a sample lesson
- An agent-collection folder with a starter agent
- A README explaining concepts

### 9. Agent Discovery Unification
**Effort:** 2-3 days
**Impact:** Resolves confusing dual-discovery model

Two agent discovery mechanisms now exist:
1. Workspace-scoped: agents from `agent-collection` folders
2. Global: agents from `agent-registry`

Define and document:
- Precedence rules (workspace-local overrides global?)
- Visibility rules (which agents appear where?)
- A unified UI that shows both sources with clear labels

### 10. Workspace Search (C)
**Effort:** 2-3 days
**Impact:** Basic expectation for content platforms

`workspace_search` exists as an agent tool. Expose it as a user-facing feature in the workspace browser toolbar.

### 11. Move Vendored JS to Package Manager (C3)
**Effort:** 2-4 days (with bun)
**Impact:** Reduces repo from ~137K JS LOC to near-zero checked-in JS

Third roast. 137,365 lines of JavaScript in the repo. Monaco editor alone is 37,016 lines. Mermaid is duplicated 8 times across storage-site builds.

Use bun to manage dependencies. Output to `static/dist/`. Add `node_modules/` to `.gitignore`. This will dramatically reduce clone times and git operation speed.

### 12. Complete Error Type Unification
**Effort:** 3-5 days
**Impact:** Consistent error responses across all 41 crates

`common::error::Error` and `ApiError` exist. Not all crates use them. Complete the migration:
- Replace all `(StatusCode, String)` returns with `ApiError`
- Ensure every HTTP error includes `request_id`
- Add `IntoResponse` impl for `ApiError` if not already present

---

## Tier 3 — This Quarter (Medium Impact, Significant Effort)

### 13. Agent Agentic Loop (C)
**Effort:** 3-5 days
**Impact:** Makes agents functional beyond chat

Carried from March 22. The framework (tools, discovery, matching, export) is extensive. The execution loop (detect tool call → execute → feed result back → repeat) is the missing piece. Without it, agents are fancy chat interfaces.

### 14. OpenAPI Specification (C)
**Effort:** 1 week
**Impact:** Enables external integrations, documents 41-crate API surface

With 41 crates exposing routes, the API surface is large and undocumented. Consider `utoipa` for derive-macro-based generation.

### 15. Tenant Management UI
**Effort:** 3-5 days
**Impact:** Makes multi-tenant usable by non-developers

`tenant_admin.rs` exists in workspace-manager. Does it have a UI? If not, tenant management is API-only, which means the multi-tenant feature is invisible to users.

### 16. Federation Demo Setup
**Effort:** 2-3 days
**Impact:** Makes federation demonstrable

Federation is a strong differentiator but hard to demo. Create:
- A `docker-compose.yml` with two AppKask instances pre-configured as federation peers
- A script that populates both instances with sample content
- Documentation showing the federation flow

### 17. Background Job System (C)
**Effort:** 1-2 weeks
**Impact:** Reliability for long-running operations

Still `tokio::spawn` with DashMap. No persistence, no retry, no concurrency limits. Less urgent now that the architecture is cleaner, but still needed for production reliability.

---

## Tier 4 — Strategic

### 18. PostgreSQL Support
**Effort:** 2-3 weeks
**Impact:** Removes SQLite ceiling

The DB trait migration made this feasible. Implement `db-postgres` crate. The business logic doesn't change — only the persistence layer.

### 19. Plugin System via Agent Definitions
**Effort:** 2-4 weeks
**Impact:** Community growth

Agent definitions as markdown are already a plugin format. Deepen this: custom tool definitions, event hooks, agent marketplace.

### 20. MCP Server Completion
**Effort:** 1-2 weeks
**Impact:** AI ecosystem integration

The scaffold exists (media-mcp). The workspace tools exist (agent-tools). Wire them together via MCP protocol. This enables Claude Desktop and any MCP client to interact with AppKask workspaces.

---

## Priority Matrix

```
                    HIGH IMPACT
                        |
  [1-3. Fix regressions]  [4. CI]  [8. Demo WS]
  [5. Split main.rs]      [9. Agent unify]
                        |
  LOW EFFORT ———————————+——————————————— HIGH EFFORT
                        |
  [7. Clean worktrees]  [6. Docker]   [18. PostgreSQL]
  [11. Vendored JS]     [13. Agent loop]
                        |              [19. Plugins]
                    LOW IMPACT
```

**The critical path:**

1. Fix regressions (#1-3) — **today** — restore clean builds
2. CI (#4) — **this week** — prevent future regressions
3. Split main.rs (#5) — **this week** — prevent next god file
4. Docker (#6) — **next week** — make AppKask installable
5. Demo workspace (#8) — **next week** — make AppKask usable

These five items, in order, would close the gap between "impressive architecture" and "usable product."

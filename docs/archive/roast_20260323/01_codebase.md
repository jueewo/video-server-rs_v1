# 01 — Codebase

## Metrics Snapshot

| Metric | March 22 | March 23 | Trend |
|--------|----------|----------|-------|
| Workspace crates | 37 | 41 | Growing (db, db-sqlite, agent-registry, workspace-processors) |
| Rust source files | ~495 | 199 (deduplicated) | Previous count included generated/template; real count is lower |
| Handwritten Rust LOC | ~63,000 | ~68,400 | +5,400 lines in one day |
| Askama templates | ~340 | 398 | +58 templates |
| Test annotations | ~20 | 267 | 13x increase |
| Tests passing | ~20 | 82 | Real passing count |
| Tests failing | 0 | 5 (doc-test compile failures) | Regression |
| Compiler warnings | 0 | 2 + build failure | Regression |
| TODO/FIXME comments | 3 | 26 across 7 files | Growth (mostly stubs) |
| Largest file | workspace-manager/lib.rs (6,030) | main.rs (1,781) | Dramatically better |
| Migrations (total) | Not tracked | 41 (6 active, 35 archived) | Mature system |

## Architecture: What's Good

### 1. The Workspace-Manager Split Is Excellent

This was the #1 structural complaint for two consecutive roasts. It's now done, and done well:

**Before (March 22):** One 6,030-line `lib.rs` handling workspace CRUD, file browsing, file editing, folder types, agent discovery, AI context, agent export.

**After (March 23):** 17 focused modules:
- `workspace_crud.rs` — workspace create/read/update/delete
- `file_browser.rs` — browsing and navigation
- `file_editor.rs` — file editing operations
- `file_ops.rs` — file manipulation primitives
- `agent_handlers.rs` — agent API endpoints
- `folder_type_handlers.rs` — folder type management
- `folder_type_registry.rs` — type discovery and registration
- `course_handlers.rs` — course-specific logic
- `presentation_handlers.rs` — presentation logic
- `site_handlers.rs` — site generation handlers (1,304 LOC — watch this one)
- `pages.rs` — page rendering (1,076 LOC)
- `publishing.rs` — publication workflows
- `tenant_admin.rs` — multi-tenant administration
- `workspace_access.rs` — access control integration
- `workspace_config.rs` — configuration handling
- `helpers.rs` — shared utilities
- `lib.rs` — now just re-exports and state definition

This is textbook modularization. Each file has a clear responsibility. The state struct uses trait objects (`Arc<dyn Repository>`), not concrete types. Well done.

### 2. The DB Trait Migration Is a Major Win

Commit `8d0137b` removed sqlx from 12+ consumer crates. The architecture is now:

```
crates/db/       → Trait definitions (zero DB driver deps)
crates/db-sqlite/ → SqliteDatabase implementing all traits
```

Domains covered: agents, api_keys, federation, git_providers, llm_providers, media, publications, user_auth, vaults, workspaces, access_control, access_codes, access_groups.

Consumer crates take `Arc<dyn SomeRepository>` — they don't know or care about SQLite. This is the single biggest architectural improvement across all three roasts. It enables:
- Future PostgreSQL support without touching business logic
- Proper unit testing with mock repositories
- Clear separation between domain logic and persistence

### 3. Test Count Jumped Significantly

From ~20 to 267 test annotations, with 82 actually passing. The distribution:
- `common`: 54 tests (largest suite)
- `media-core`: 17 tests
- `federation`: 11 tests
- `authz_integration`: comprehensive integration test file

The integration test (`tests/authz_integration.rs`) is notable — it tests session auth, media CRUD, owner/non-owner access patterns, and request-id propagation. This is the kind of test that catches real bugs.

### 4. Multi-Tenant Isolation Is a Real Feature

Commit `ba31688` added `tenant_id` to media_items, federation_peers, and remote_media_cache. All listing, search, and insert queries are now tenant-scoped. This isn't just a column — it's a design decision that says "this platform can serve multiple organizations on one instance."

Smart choice: slug-based lookups remain global (intentional, documented). This means sharing via access codes works across tenant boundaries, which is the right behavior for the consultant use case.

### 5. Access Control Is Well-Designed

The 4-layer model (Public, AccessKey, GroupMembership, Ownership) with permission hierarchy (Admin > Delete > Edit > Download > Read) is clean. It has:
- Audit logging (`audit.rs`)
- Type-safe queries via the repository pattern
- Integration tests that verify ownership boundaries

## Architecture: What's Concerning

### 1. main.rs Is the New God File

1,781 lines and growing (was 1,684 on March 22). It's now the single largest file in the project. It handles:
- AppConfig and DeploymentConfig structs
- Production secret validation
- OIDC setup
- Session management
- CORS configuration
- Rate limiting (3 tiers)
- OpenTelemetry setup
- Federation configuration
- Route wiring for 41 crates
- Static file serving
- Middleware stacking

You fixed the workspace-manager god file by splitting it into 17 modules. main.rs needs the same treatment:
- `config.rs` — AppConfig, DeploymentConfig, environment parsing
- `security.rs` — production validation, OIDC, session setup
- `middleware.rs` — CORS, rate limiting, request-id
- `routes.rs` — route composition from all crates
- `main.rs` — just `#[tokio::main]`, create pool, create states, merge routers, bind server

### 2. The webdav Crate Doesn't Compile

`Pool<Sqlite>: ApiKeyRepository` trait not implemented. This means `cargo build --workspace` fails. A crate that doesn't compile is worse than a crate that doesn't exist — it blocks workspace-wide builds, tests, and CI (if you had CI).

**Fix immediately:** Either implement the missing trait or add `#[cfg(feature = "webdav")]` to exclude it from default workspace builds.

### 3. Five Doc-Test Compile Failures

All in the `access-control` crate. Doc tests that reference types or methods that have changed during the DB trait migration. These are the kind of tests that a CI pipeline would catch on every push.

### 4. Two Compiler Warnings

Unused variables in `bpmn-simulator-processor`. Small, but you had zero warnings as of March 22. The broken-windows effect: if two warnings are acceptable, four will be next.

### 5. site_handlers.rs Is Growing Fast

At 1,304 LOC, `workspace-manager/src/site_handlers.rs` is already the second-largest file in workspace-manager. It risks becoming the next god file. Monitor this — if it passes 1,500 LOC, split it.

### 6. Stale Worktrees

Three stale Claude worktrees (`.claude/worktrees/`) with duplicated files. These add noise to searches and inflate disk usage. Clean them up.

## Code Smells

| Smell | Location | Severity | Change from March 22 |
|-------|----------|----------|---------------------|
| God file | main.rs (1,781 LOC) | High | Inherited from workspace-manager |
| Build failure | webdav crate | **Critical** | New regression |
| Doc-test failures | access-control (5) | Medium | New regression |
| Unused warnings | bpmn-simulator-processor (2) | Low | New regression |
| No CI pipeline | Project-wide | High | Unchanged (3rd roast) |
| No Docker | Project-wide | High | Unchanged (3rd roast) |
| Vendored JS | 137K LOC in git | Medium | Unchanged (3rd roast) |
| Stale worktrees | .claude/worktrees/ (3) | Low | New |
| TODO stubs | media-cli (8), media-mcp (6), site processors (6) | Low | Grew from 3 to 26 |
| No OpenAPI spec | Project-wide | Medium | Unchanged |

## New Crate Assessment

### db + db-sqlite (combined ~2,500+ LOC)

**Quality:** Excellent. Clean trait-per-domain design. The sqlite implementation uses sqlx correctly with proper error mapping to `DbError`.

**Pattern:** Each domain gets its own trait file in `db/` and its own impl file in `db-sqlite/`. This scales well — adding PostgreSQL would mean a `db-postgres` crate implementing the same traits.

### agent-registry

**Quality:** Good. Adds a global agent workforce concept on top of the per-workspace agent-collection discovery.

**Concern:** Two agent discovery mechanisms now exist (workspace-scoped via `agent-collection` folders and global via `agent-registry`). The relationship between them should be documented.

### workspace-processors/*

**Quality:** Framework in place. Individual processors (bpmn-simulator, static-site) have significant TODO counts.

**Concern:** The `static-site` processor has 6 TODOs for framework build support. The `bpmn-simulator` processor has 3 TODOs and the 2 compiler warnings. These are scaffolds, not implementations.

## Verdict

This is the best single-cycle improvement across all three roasts. The workspace-manager split and DB trait migration addressed the top two structural concerns. Test coverage went from embarrassing to respectable. Multi-tenant isolation and federation hardening are real product features, not just code movement.

But: you introduced three regressions (build failure, doc-test failures, warnings) that a CI pipeline would have caught. The webdav build failure is particularly bad — it means no one can do a clean `cargo build --workspace`. And main.rs quietly became the new largest file while you were fixing workspace-manager. Apply the same discipline that produced the workspace-manager split to main.rs before it reaches 2,000 lines.

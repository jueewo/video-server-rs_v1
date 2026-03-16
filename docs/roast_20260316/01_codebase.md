# 01 - Codebase & Code Structure Roast

## The Numbers

| Metric | Value |
|--------|-------|
| Rust source files | 139 (active, excl. archive/worktrees) |
| Lines of Rust | ~55,000 (active) |
| Workspace crates | 34 |
| Askama templates | 331 HTML files |
| Integration test files | 1 (`authz_integration.rs`) |
| Unit tests | 0 |
| Passing tests | 6/11 (55%) |
| Largest file | `workspace-manager/src/lib.rs` (4,978 lines) |
| `main.rs` | 1,538 lines |

---

## What's Good

### Modular Monolith Done Right (Mostly)
The Cargo workspace with 34 crates is a genuinely good architectural choice. Each crate has a clear domain boundary (`access-control`, `media-manager`, `vault-manager`). The `pub fn foo_routes() -> Router` convention means wiring is explicit and discoverable. This is miles ahead of a single-crate spaghetti ball.

### The Dual-Use Pattern is Clever
The `FolderTypeRenderer` trait that lets a crate run embedded (inside workspace chrome) OR standalone (own Router) is an elegant abstraction. It's the kind of design that lets you ship one thing but deploy it two ways — and it actually works.

### Storage Architecture is Sound
The vault-based nested storage (`storage/vaults/{vault_id}/media/{type}/`) with clear UserStorageManager methods is well-designed. Media isolation per vault, symmetric thumbnail paths, and the `find_media_file()` / `find_thumbnail()` pattern are clean.

### Access Control is Production-Grade
The 4-layer access model (Public > AccessCode > Group > Owner) with audit logging is genuinely sophisticated. Most solo projects would have a boolean `is_admin` field. You have per-resource permission hierarchies with audit trails. That's enterprise-grade thinking.

### HLS Pipeline is Non-Trivial
8-stage transcoding with WebSocket progress, multi-quality adaptive bitrate, and auto quality selection — this is real video infrastructure, not a toy.

---

## What's Bad

### Test Coverage is Effectively Zero
This is the single biggest red flag. 55,000 lines of Rust, 0 unit tests, 1 integration test file with 5/11 tests failing. For a "production-ready" platform handling file uploads, access control, and financial data (billing/licensing via Casdoor), this is a liability.

**Specific risks:**
- The access control system has no tests. A permission bypass bug would be invisible.
- Upload validation (MIME checking, path traversal protection) is untested.
- The HLS transcoding pipeline has no tests for edge cases (corrupt files, zero-byte uploads, race conditions).
- Database migrations are untested — you're relying on manual `sqlite3` commands.

**The failing tests tell a story:** `upload_returns_401_without_session` expects 401 but gets 405 — your route structure has drifted from what the tests expect. Tests that exist are stale.

### God Files
Several files are too large and do too much:

| File | Lines | Problem |
|------|-------|---------|
| `workspace-manager/src/lib.rs` | 4,978 | Entire crate in one file |
| `video-manager/src/lib.rs` | 1,811 | Mixed concerns |
| `media-manager/src/upload.rs` | 1,700 | Could split by media type |
| `main.rs` | 1,538 | Config, state, routes, middleware all in one |
| `access-codes/src/lib.rs` | ~1,200 | Everything in lib.rs |

The `workspace-manager` is the worst offender — nearly 5,000 lines in a single `lib.rs`. This crate handles workspace CRUD, file browsing, folder type management, access control, and rendering. That's at least 4-5 modules crammed into one file.

### Inconsistent Module Organization
Some crates are well-structured (`media-manager` has `routes.rs`, `models.rs`, `upload.rs`, `serve.rs`, `list.rs` — good). Others dump everything into `lib.rs` (`access-codes`, `vault-manager`, `workspace-manager`). There's no clear convention being followed.

### main.rs is a Configuration Monolith
1,538 lines of setup code. Config loading, database init, OIDC setup, session config, rate limiting, route wiring, middleware stacking, and startup banners — all in one function. This should be broken into `config.rs`, `routes.rs`, `middleware.rs`, and `startup.rs` at minimum.

### Archive and Worktree Clutter
The repo has archive directories and `.claude/worktrees/` that inflate line counts and create confusion. Legacy code (`image-manager`, `document-manager`, `media-hub`) still lives in `archive/` — these should be in git history, not the working tree.

### Vendor Libraries Committed to Repo
`static/vendor/` contains Monaco editor, React, Reveal.js, bpmn-js, Excalidraw, Mermaid, KaTeX — hundreds of megabytes of vendored JS. This should be managed by a package manager or CDN, not committed to git. It makes cloning slow, diffs noisy, and updates manual.

### Error Handling is Inconsistent
Some handlers return `(StatusCode, Json)` tuples, others use `Result<impl IntoResponse>`, some use `.map_err(|e| ...)` inline. There's no unified error type across crates. A custom `AppError` enum with `IntoResponse` impl would clean this up significantly.

### SQL Queries are Inline Strings
Raw SQL strings scattered throughout handlers. No query builder, no typed queries beyond what sqlx compile-time checking provides. The dynamic query building in `update_media_item` (string concatenation for SET clauses) is a maintenance hazard.

---

## What's Ugly

### Migration System is Fragile
The doc says "SQLite migrations are NOT auto-applied. Apply manually." But MEMORY.md says "sqlx::migrate!() auto-applies anything in migrations/." Which is it? The system has legacy numbered migrations (001-023), timestamp migrations, and an `applied/` archive. A new developer would have no idea what state their database is in.

### Feature Flags Create Combinatorial Complexity
`full = ["media", "course", "bpmn", "apps"]` means 16 possible feature combinations. Is `course` without `media` even valid? Are there compile errors for certain combinations? Without tests, nobody knows.

### Dead Code and Phantom Modules
The common crate has empty `db`, `handlers`, `routes` modules (per MEMORY.md). The tag system was partially removed but tag search is still alive. There are likely more phantom exports and dead branches lurking.

### No CI/CD
No `.github/workflows/`, no `Justfile`, no `Makefile` for common dev operations. Building, testing, linting, and deploying are entirely manual and undocumented beyond CLAUDE.md.

---

## Severity Summary

| Issue | Severity | Fix Effort |
|-------|----------|------------|
| Zero test coverage | **Critical** | High (ongoing) |
| Failing existing tests | **High** | Low (fix or delete) |
| God files (workspace-manager) | **High** | Medium |
| main.rs monolith | **Medium** | Medium |
| No CI/CD | **High** | Low |
| Inconsistent error handling | **Medium** | Medium |
| Vendored JS libraries | **Medium** | Low-Medium |
| Migration system confusion | **Medium** | Low |
| Archive clutter in working tree | **Low** | Low |

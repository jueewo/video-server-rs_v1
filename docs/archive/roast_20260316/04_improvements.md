# 04 - Suggestions for Improvement

Ranked by impact-to-effort ratio. Do the top ones first.

---

## Tier 1: Do This Week (High Impact, Low Effort)

### 1. Fix or Delete the Failing Tests
**Why:** 5/11 tests fail. Stale tests are worse than no tests — they teach you to ignore test failures.
**Action:** Fix the 5 failing tests or delete them. Then add a `cargo test` step to a pre-commit hook or CI so they never rot again.
**Effort:** 2-4 hours

### 2. Add a CI Pipeline
**Why:** No CI means every push is a gamble. You don't know if it compiles on a clean machine, if tests pass, or if clippy is happy.
**Action:** Add a `.github/workflows/ci.yml` with: `cargo check`, `cargo clippy`, `cargo test`, `cargo fmt --check`. Even without tests, catching compilation errors on every push is worth it.
**Effort:** 1-2 hours

### 3. Remove the Archive Directory from Working Tree
**Why:** `archive/` contains legacy code (image-manager, document-manager, media-hub) that inflates the repo and confuses exploration. It's in git history forever — you don't need it in the working tree.
**Action:** `git rm -r archive/` — if you ever need it, `git log` has it.
**Effort:** 30 minutes

### 4. Move Vendored JS to a Package Manager
**Why:** `static/vendor/` with Monaco, React, Reveal.js, bpmn-js, etc. makes the repo enormous, diffs noisy, and updates manual.
**Action:** Use a `package.json` with bun, run `bun install`, add `node_modules/` to `.gitignore`, and copy needed files during build. Or use a CDN with SRI hashes for the truly static ones.
**Effort:** 2-4 hours

---

## Tier 2: Do This Month (High Impact, Medium Effort)

### 5. Split the God Files
**Priority targets:**

| File | Lines | Split Into |
|------|-------|-----------|
| `workspace-manager/src/lib.rs` | 4,978 | `routes.rs`, `handlers.rs`, `models.rs`, `file_browser.rs`, `folder_types.rs` |
| `main.rs` | 1,538 | `config.rs`, `routes.rs`, `middleware.rs`, `startup.rs` |
| `access-codes/src/lib.rs` | ~1,200 | `models.rs`, `handlers.rs`, `routes.rs` |

**Why:** Large files slow down comprehension, make merge conflicts worse, and hide bugs. The workspace-manager at 5K lines is the most urgent — it's the core navigation crate.
**Effort:** 1-2 days per file

### 6. Unified Error Type
**Why:** Handlers use a mix of `(StatusCode, Json)`, `Result<impl IntoResponse>`, and inline `.map_err()`. A unified error type makes error handling consistent and enables global error logging.
**Action:**
```rust
pub enum AppError {
    NotFound(String),
    Unauthorized,
    Forbidden(String),
    BadRequest(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError { ... }
```
Use this across all crates via the `common` crate.
**Effort:** 2-3 days

### 7. Add Core Tests
Don't try to get to 80% coverage overnight. Add tests where they matter most:

| Area | What to Test | Why |
|------|-------------|-----|
| Access control | Each layer grants/denies correctly | Security-critical |
| Upload validation | MIME checks, path traversal, size limits | Security-critical |
| Slug generation | Uniqueness, sanitization, edge cases | Data integrity |
| Media serving | Access code validation, fallback chains | Core functionality |
| Storage paths | Vault paths are correct, no path traversal | Security-critical |

**Effort:** 1 week for the critical paths

### 8. Simplify the Upload Form
**Current:** 12+ fields (title, slug, description, is_public, media_type, category, tags, vault_id, group_id, transcode_for_streaming, keep_original, file)
**Better:** 3 fields (file, title, vault) with an "Advanced options" expandable section for the rest. Auto-detect media type, auto-generate slug, default to private, default to keep original + auto-transcode.
**Effort:** 1 day

---

## Tier 3: Do This Quarter (Medium Impact, Higher Effort)

### 9. Global Search (Ctrl+K Omnibar)
**Why:** Users can't find things. Each section has its own search. A global omnibar that searches media, workspaces, folders, and documents would transform discoverability.
**Action:** Add a search endpoint that queries across tables, with a floating modal triggered by Ctrl+K. Alpine.js + HTMX can handle this without React.
**Effort:** 1 week

### 10. One-Click Sharing
**Why:** The current sharing flow requires navigating to access code management. It should be a "Share" button on every media item and folder.
**Action:** Add a share modal that creates an access code, generates the URL, and offers "Copy link." Show existing codes for the resource.
**Effort:** 3-5 days

### 11. Setup Wizard
**Why:** New users have no idea what to do after first login.
**Action:** On first login (or when no workspaces exist), show a guided flow:
1. "Welcome! Let's set up your workspace."
2. Create workspace + first folder
3. Upload first file
4. Show the share button
**Effort:** 3-5 days

### 12. Audit Log UI
**Why:** You have a sophisticated audit backend (`access_audit_log`) with zero UI. For regulated industries (Persona 3), this is table stakes.
**Action:** Add `/admin/audit` with filterable log table (user, resource, action, timestamp, result). Export to CSV.
**Effort:** 3-5 days

### 13. Background Job System
**Why:** HLS transcoding, PDF thumbnails, and image conversion all spawn `tokio::spawn` tasks with no visibility, retry logic, or failure handling. If the server restarts mid-transcode, the job is lost.
**Action:** Add a `jobs` table (id, type, status, payload, created_at, started_at, completed_at, error). Process jobs on startup (resume interrupted ones). Show job status in admin UI.
**Effort:** 1-2 weeks

---

## Tier 4: Strategic Investments (High Effort, High Long-Term Value)

### 14. API Documentation (OpenAPI/Swagger)
**Why:** If you want external developers, satellite apps, or the MCP server to integrate, they need documented API contracts. Currently the API is discovered by reading Rust source code.
**Action:** Use `utoipa` or `aide` to generate OpenAPI specs from your handler annotations. Serve Swagger UI at `/api/docs`.
**Effort:** 1-2 weeks

### 15. Migrate to WAL Mode + Connection Pooling for SQLite
**Why:** SQLite's default journal mode blocks readers during writes. WAL (Write-Ahead Logging) allows concurrent reads with writes.
**Action:** `PRAGMA journal_mode=WAL;` on startup (may already be set via sqlx). Add connection pool monitoring. Consider read replicas for heavy read paths.
**Effort:** 1-2 days (if not already done)

### 16. Plugin/App SDK
**Why:** The folder-type system is your extensibility story. But creating a new app currently requires: writing a Rust crate, implementing `FolderTypeRenderer`, wiring it into `main.rs`, and recompiling. That's not a plugin system — that's a contribution requirement.
**Action:** Define a plugin interface that allows apps to be loaded dynamically (WASM, separate processes, or at minimum a well-documented crate template with a `create-app` CLI command).
**Effort:** 2-4 weeks

### 17. Observability Dashboard
**Why:** You have OpenTelemetry integration but no built-in visibility. For self-hosted deployments, users need to see system health without setting up Grafana.
**Action:** Add `/admin/system` with: active users, storage usage per vault, transcoding queue length, request rate, error rate. Pull from your existing tracing data.
**Effort:** 1 week

---

## What NOT to Improve Right Now

- **Don't add PostgreSQL support.** SQLite is fine for your current scale. Adding Postgres doubles your testing surface for no current customer demand.
- **Don't build mobile apps.** The web UI works on mobile (barely). Fix the responsive design before building native apps.
- **Don't build real-time collaboration.** This is a fundamental architecture change (CRDTs, operational transforms). It's a different product.
- **Don't add more verticals.** You have 7 feature areas. Make 3 of them excellent before adding an 8th.
- **Don't optimize for scale.** 50 concurrent users is your ceiling. Optimize for developer experience and feature completeness, not throughput.

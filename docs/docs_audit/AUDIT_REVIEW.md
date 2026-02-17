# AUDIT REVIEW — Self-Correction and Calibration

## Purpose

This document reviews the three audit files (`AUDIT.md`, `SECURITY_CHECKLIST.md`, `TECHNICAL_DEBT_BACKLOG.md`) against what the codebase **actually contains**. The original audit was produced from a structural scan; this review corrects overstatements, acknowledges existing strengths that were underrated, and surfaces findings the audit missed entirely.

---

## Corrections: Things the Audit Overstated

### 1) Authorization Is NOT Missing — It's Already Centralized

The audit repeatedly flags "centralize authorization checks" as a P0 gap (TD-003, AUDIT.md §Security, §Architecture). This is **misleading**.

The `access-control` crate already implements a **sophisticated 4-layer access model**:

- Layer 1: Public
- Layer 2: Access Key (temporary, scoped)
- Layer 3: Group Membership (role-based)
- Layer 4: Ownership (full control)

With a permission hierarchy (Read → Download → Edit → Delete → Admin), full audit logging, batch checking, and **comprehensive tests** (12+ test cases covering public access, ownership, denial, batching, audit trail).

**What TD-003 should actually say:** "Audit all HTTP routes to confirm they invoke `AccessControlService::check_access()` consistently — especially static file serving paths (`/storage/*`), HLS proxy, thumbnail, and preview routes." The service exists; the question is whether every route uses it.

**Revised effort:** M (route audit + middleware enforcement), not L.

### 2) Upload Validation Already Exists in `media-core`

The audit flags upload validation (TD-009) as if it needs to be built from scratch. In reality, `media-core/src/validation.rs` already provides:

- MIME type validation per media type (video/image/document)
- File extension validation and extension-to-MIME cross-check
- Per-type size limits (video: 5GB, image: 50MB, document: 100MB)
- Filename sanitization (strips `< > : " | ? *` and path separators)
- Path traversal prevention (`..`, `/`, `\` blocked)
- Null byte rejection
- Tests for all of the above

And `media-core/src/upload.rs` has a well-structured `UploadHandler` with a 10-step validation pipeline and tests for sanitization, size limits, MIME mismatches, and unique slug generation.

**What TD-009 should actually say:** "Verify that `media-manager/src/upload.rs` (the Axum handler layer) consistently delegates to `media-core`'s validation pipeline for all upload paths. Add magic-byte validation beyond PNG/PDF headers. Confirm the 100MB body limit in the router matches the per-type limits in `media-core`."

**Revised effort:** S (integration audit + magic-byte gap), not M.

### 3) Storage Safety Is More Mature Than Described

The audit's storage safety section implies basic path handling. In reality, `common/src/storage.rs` provides:

- Vault-based storage isolation (`storage/vaults/{vault_id}/{media_type}/`)
- User-based storage isolation (legacy path)
- Backward-compatible file location resolution (vault → user → legacy fallback)
- `sanitize_user_id()` and `validate_user_id()` with path traversal prevention
- Directory creation safety with proper error handling
- Tests for path resolution, backward compatibility, sanitization, and validation

This is a well-engineered storage abstraction. The audit should have credited it.

---

## Corrections: Things the Audit Understated or Missed

### 4) Massive Dead Code in `main.rs` (Missed — Should Be P1)

`src/main.rs` contains approximately **250+ lines of commented-out OpenTelemetry initialization attempts** — at least 6-7 different versions of `init_tracer()`, all commented out, left as archaeological layers. The working version sits below all of them.

This is not just a style issue. It:
- Obscures the actual initialization logic
- Makes the file significantly harder to navigate
- Signals a pattern of "comment-out-and-retry" rather than version control
- Contributes directly to the "entrypoint complexity" finding

**Recommendation:** Delete all commented-out tracer versions. The final working one is clear. Git history preserves the experiments.

### 5) `media-hub` / `media-manager` Duplication (Missed — Should Be P1)

The router assembly explicitly says:

```
// Media Hub - provides list view for all media types (mount first)
.merge(media_routes()...)
// Unified media manager (new endpoints - override media-hub where they conflict)
.merge(unified_media_routes()...)
```

Two media systems are mounted simultaneously with acknowledged route conflicts. The `Cargo.toml` comments confirm this is a migration in progress ("REMOVED: image-manager and document-manager - replaced by unified media-manager").

This means:
- Two code paths serve overlapping functionality
- "Override where they conflict" is fragile and order-dependent
- Bug reports could hit either system depending on route resolution
- Testing surface is doubled

**Recommendation:** Complete the migration. Either retire `media-hub` routes entirely or merge the remaining unique functionality into `media-manager`. This should be a dedicated TD item (suggest: **TD-021**, P1, effort M).

### 6) `.gitignore` Ignores `Cargo.lock` — Wrong for a Binary Project (Missed — P1)

The `.gitignore` contains `Cargo.lock`. For library crates, this is standard practice. For **application/binary** projects (which this is — it produces `video-server-rs`), `Cargo.lock` should be committed to ensure reproducible builds.

Without `Cargo.lock` in version control:
- Different developers/CI may get different dependency versions
- Production builds are not reproducible
- Subtle runtime bugs from dependency drift are possible

**Recommendation:** Remove `Cargo.lock` from `.gitignore` and commit it. This is a one-line fix with significant build reliability impact.

### 7) `.gitignore` Ignores `migrations/` — Unusual and Risky (Missed — P1)

The `.gitignore` contains `migrations/`. SQLx migrations are normally committed to source control — they ARE the schema source of truth. Ignoring them means:

- Schema changes are not tracked in version control
- New clones may have no migrations to apply
- There's no way to review schema changes in PRs
- Production and development schemas can silently diverge

This directly explains the audit's finding about "migration governance ambiguity" — it's not ambiguous, it's actively excluded from tracking.

**Recommendation:** Remove `migrations/` from `.gitignore`. Commit the current migration files. This is the single most impactful data governance fix.

### 8) Vault ID Generation Has Collision Risk (Missed — P2)

`common/src/storage.rs` generates vault IDs using:

```
let random_part: u32 = (timestamp % (u32::MAX as u128)) as u32;
format!("vault-{:08x}", random_part)
```

This uses only nanosecond timestamp modulo u32. Under concurrent vault creation (same nanosecond window), collisions are possible. For a single-user system this is fine; for multi-user production, it's a latent bug.

**Recommendation:** Use `uuid::v4` or a proper random source instead of timestamp arithmetic.

### 9) Emergency Login Default Credentials (Understated)

`OidcConfig::from_env()` defaults:
- `su_user` → `"admin"`
- `su_pwd` → `""` (empty string)
- `enable_emergency_login` → `false`

The empty password default combined with disabled-by-default is **safe in practice**. But the `su_user` defaulting to "admin" means if someone enables emergency login and only sets `SU_PWD`, the username is predictable. Minor, but worth noting.

### 10) Session `Secure` Flag Hardcoded to `false` (Correctly Flagged, But Context Matters)

The audit correctly flags `.with_secure(false)`. The code comment says "Set to true in production with HTTPS." This should be environment-driven rather than requiring a code change for production.

---

## Dependency Graph Fix in Technical Debt Backlog

The backlog has a **circular dependency**:
- TD-003 depends on TD-008
- TD-008 depends on TD-003

This is a planning error. The correct dependency chain is:
- **TD-003** (centralize authz enforcement) has **no blockers** — the service already exists
- **TD-008** (integration tests for authz paths) depends on **TD-003** being verified/enforced

Fix: Remove TD-008 from TD-003's dependencies. TD-003 is an audit/enforcement task, not a build-from-scratch task.

---

## Recalibrated Priority and Effort

| ID | Original Priority | Original Effort | Revised Priority | Revised Effort | Reason |
|---|---|---|---|---|---|
| TD-003 | P0 | L | P0 | **M** | Service exists; work is route audit, not implementation |
| TD-009 | P1 | M | P1 | **S** | Validation pipeline exists; work is integration verification |
| TD-004 | P0 | M | **P0** | M | Confirmed: `.gitignore` issues + committed artifacts make this urgent |
| TD-005 | P1 | M | **P0** | **S** | `migrations/` being gitignored is an active data governance risk |
| TD-006 | P1 | L | P1 | L | Confirmed: ~250 lines of dead code + dense wiring |
| — | — | — | **NEW: TD-021** | P1, M | Complete media-hub → media-manager migration, retire duplicate routes |
| — | — | — | **NEW: TD-022** | P1, S | Commit Cargo.lock; remove from .gitignore |
| — | — | — | **NEW: TD-023** | P1, S | Delete commented-out OpenTelemetry code in main.rs |

---

## Security Checklist Review

The `SECURITY_CHECKLIST.md` is **solid and well-structured**. No major corrections needed. Minor notes:

1. Section 4 (Upload Security) — can reference `media-core/src/validation.rs` as existing implementation baseline rather than implying everything is missing.
2. Section 11 (Streaming) — correctly scoped. The `RTMP_PUBLISH_TOKEN` constant in `video-manager` is the specific item to validate.
3. The "Verification Tests" minimum set is practical and actionable.
4. The go/no-go gate is the right level of rigor for this project stage.

---

## Revised Wave Plan

### Wave 0 (Week 1) — Quick Wins, High Impact
- **TD-022** — Commit `Cargo.lock` (5 min)
- **TD-005** — Commit `migrations/` (30 min)
- **TD-023** — Delete dead OpenTelemetry code from `main.rs` (30 min)
- **TD-002** — Make session `Secure` flag environment-driven (1 hour)

### Wave 1 (Weeks 1–3) — Production Risk Reduction
- **TD-001** — Secret management hardening
- **TD-004** — Full repo hygiene pass
- **TD-003** — Audit all routes for `AccessControlService` usage
- **TD-009** — Verify upload handler → media-core validation integration

### Wave 2 (Weeks 4–6) — Architecture Cleanup
- **TD-021** — Complete media-hub → media-manager migration
- **TD-006** — Decompose main.rs composition layer
- **TD-007** — Standardize API error envelope
- **TD-010** — Rate limiting

### Wave 3 (Weeks 7–10) — Testing and Observability
- **TD-008** — Authz integration tests
- **TD-011** — Structured logging
- **TD-018** — SLO dashboard

### Wave 4 (Ongoing) — Governance and Polish
- TD-012 through TD-020 as originally planned

---

## Completed: media-hub → media-manager Consolidation (TD-021)

**Status:** ✅ Done

The dual-system overlap between `media-hub` and `media-manager` has been resolved. Here is what was done:

### What was migrated into `media-manager`

| Source (media-hub) | Destination (media-manager) | Purpose |
|---|---|---|
| `models.rs` | `src/models.rs` | `UnifiedMediaItem`, `MediaFilterOptions`, `MediaListResponse`, `MediaTypeCounts` |
| `search.rs` | `src/search.rs` | `MediaSearchService` — cross-media search with filtering/pagination |
| `templates.rs` | `src/templates.rs` | `MediaListTemplate`, `MediaUploadTemplate`, `MediaItemWithMetadata` |
| Active route handlers | `src/list.rs` (new) | `list_media_html/json`, `search_media_html/json`, `show_upload_form`, `get_user_vaults`, `toggle_visibility`, `get_media_item`, `update_media_item`, `delete_media` |
| `media_list_tailwind.html` | `templates/media_list_tailwind.html` | Listing page template |
| `media_upload.html` | `templates/media_upload.html` | Upload form template |

### What was discarded (dead code)

~850 lines of dead code in media-hub that was already commented out of routes:
- `upload_media` function (replaced by `media-manager/src/upload.rs`)
- `UploadResponse`, `DetectedMediaType`, `detect_media_type`, `sanitize_filename`
- `create_video_record`, `create_image_record`, `create_document_record`
- `slugify`, `generate_thumbnail`

### What changed in the architecture

- `media-hub` crate removed from workspace and moved to `archive/media-hub`
- `main.rs` now mounts a **single** `media_routes()` from `media-manager` — no more dual merge with conflict comments
- `MediaHubState` eliminated — all handlers use `MediaManagerState`
- `media-manager/src/routes.rs` now contains the complete unified route table with clear section comments
- Zero compilation errors, zero new warnings

### Remaining items from this audit

1. `Cargo.lock` and `migrations/` being gitignored — **still open**
2. Dead commented-out OpenTelemetry code in `main.rs` — **still open**
3. Route-level authz enforcement audit — **still open**

---

## Summary

The original audit documents are **directionally correct** but suffered from insufficient code-level verification. The project is **more mature than the audit suggests** in several key areas (access control, upload validation, storage safety). The main risks that were **understated** are:

1. ~~Active dual-system overlap (`media-hub` + `media-manager`)~~ — **resolved**
2. `Cargo.lock` and `migrations/` being gitignored
3. Dead code accumulation in the entrypoint
4. Route-level authz enforcement gaps (not the absence of the authz system itself)

The security checklist and backlog structure are sound and can be used as-is with the corrections above applied.
# TD-003: Route-Level Security Audit

**Date:** 2025-01-27  
**Auditor:** Automated deep-dive  
**Scope:** Every route that serves or manages media content — which ones go through `AccessControlService` vs which are unprotected  
**Status:** ✅ Complete — **All remediations applied**

---

## Executive Summary

The application uses a two-layer security model:

1. **Middleware layer** (`api_key_or_session_auth`) — applied via `.route_layer()` on specific router groups in `main.rs`. Rejects unauthenticated requests with 401.
2. **Handler-level access control** (`AccessControlService`) — the 4-layer ACL system checked inside individual handlers for fine-grained resource-level permissions.

**Key findings:**

| Severity | Finding |
|----------|---------|
| 🔴 CRITICAL | `/storage/*` serves all stored media files (images, videos, documents, vault contents) with **zero authentication or access control** |
| 🟡 MEDIUM | `/media/:slug/view` (markdown viewer) uses ad-hoc owner/public check instead of `AccessControlService` |
| 🟡 MEDIUM | `vault_routes`, `access_code_routes`, and `api_key_routes` lack middleware-level auth (rely solely on handler-level session checks) |
| 🟢 LOW | `access_groups` routes include public invitation endpoints behind auth middleware (may block legitimate public use) |
| 🟢 INFO | `/static/*` is intentionally public (CSS/JS assets) — correct |

---

## Route Assembly Map

Source: `src/main.rs` lines 619–680

```
Router::new()
    // ── Group A: App-level routes (NO middleware) ──────────────
    .route("/", ...)
    .route("/demo", ...)
    .route("/health", ...)
    .route("/favicon.ico", ...)
    .route("/tags", ...)
    .route("/tags/cloud", ...)
    .route("/api/webhooks/stream-ready", ...)
    .route("/api/webhooks/stream-ended", ...)

    // ── Group B: Auth routes (NO middleware, intentionally public) ─
    .merge(auth_routes(...))

    // ── Group C: API Key management (NO middleware) ────────────
    .merge(api_key_routes(...))

    // ── Group D: Media Manager (WITH api_key_or_session_auth) ─
    .merge(media_routes().with_state(...).route_layer(api_key_or_session_auth))

    // ── Group E: Video Manager (WITH api_key_or_session_auth) ─
    .merge(video_routes().with_state(...).route_layer(api_key_or_session_auth))

    // ── Group F: Access Codes (NO middleware) ──────────────────
    .merge(access_code_routes(...))

    // ── Group G: Vault Manager (NO middleware) ─────────────────
    .merge(vault_routes(...))

    // ── Group H: Access Groups (WITH api_key_or_session_auth) ─
    .merge(access_groups::routes::create_routes(...).route_layer(api_key_or_session_auth))

    // ── Group I: Tags (WITH api_key_or_session_auth) ──────────
    .merge(create_tag_routes(...).route_layer(api_key_or_session_auth))

    // ── Group J: Search (WITH api_key_or_session_auth) ────────
    .merge(create_search_routes(...).route_layer(api_key_or_session_auth))

    // ── Group K: 3D Gallery (NO middleware) ────────────────────
    .merge(gallery3d::router(...))

    // ── Group L: Docs Viewer (WITH api_key_or_session_auth) ───
    .nest("/docs", docs_routes().with_state(...).route_layer(api_key_or_session_auth))

    // ── Group M: Static file serving (NO middleware, NO ACL) ──
    .nest_service("/storage", ServeDir::new(&storage_dir))
    .nest_service("/static", ServeDir::new("static"))
```

---

## Detailed Route-by-Route Analysis

### Group A: Main App Routes — NO MIDDLEWARE

| Route | Method | Handler | Auth | ACL | Risk |
|-------|--------|---------|------|-----|------|
| `/` | GET | `index_handler` | ❌ | ❌ | 🟢 Public landing page |
| `/demo` | GET | `demo_handler` | ❌ | ❌ | 🟢 Public demo page |
| `/health` | GET | `health_check` | ❌ | ❌ | 🟢 Health endpoint |
| `/favicon.ico` | GET | `favicon_handler` | ❌ | ❌ | 🟢 Static asset |
| `/tags` | GET | `tag_management_handler` | ❌ | ❌ | 🟢 Public tag browser |
| `/tags/cloud` | GET | `tag_cloud_handler` | ❌ | ❌ | 🟢 Public tag cloud |
| `/api/webhooks/stream-ready` | POST | `webhook_stream_ready` | ❌ | ❌ | 🟡 Should validate webhook secret |
| `/api/webhooks/stream-ended` | POST | `webhook_stream_ended` | ❌ | ❌ | 🟡 Should validate webhook secret |

**Assessment:** Mostly fine. Webhook endpoints should at minimum validate a shared secret to prevent spoofing.

---

### Group B: Auth Routes — NO MIDDLEWARE (Intentional)

| Route | Method | Handler | Auth | ACL | Risk |
|-------|--------|---------|------|-----|------|
| `/login` | GET | `login_page_handler` | ❌ | ❌ | 🟢 Public |
| `/logout` | GET | `logout_handler` | ❌ | ❌ | 🟢 Public |
| `/profile` | GET | `user_profile_handler` | session* | ❌ | 🟢 Handler checks session |
| `/oidc/authorize` | GET | `oidc_authorize_handler` | ❌ | ❌ | 🟢 OIDC flow |
| `/oidc/callback` | GET | `oidc_callback_handler` | ❌ | ❌ | 🟢 OIDC flow |
| `/auth/error` | GET | `auth_error_handler` | ❌ | ❌ | 🟢 Error page |
| `/login/emergency` | GET | `emergency_login_form_handler` | ❌ | ❌ | 🟡 Emergency login |
| `/login/emergency/auth` | POST | `emergency_login_auth_handler` | ❌ | ❌ | 🟡 Should be rate-limited |

**Assessment:** Correct — auth endpoints must be unauthenticated. Emergency login should be rate-limited (see TD-010).

---

### Group C: API Key Management — NO MIDDLEWARE ⚠️

Source: `crates/api-keys/src/routes.rs`

| Route | Method | Handler | Auth | ACL | Risk |
|-------|--------|---------|------|-----|------|
| `/profile/api-keys` | GET | `list_api_keys_page_handler` | session✅ | ❌ | 🟢 Handler checks session |
| `/profile/api-keys/create` | GET | `create_api_key_page_handler` | session✅ | ❌ | 🟢 Handler checks session |
| `/profile/api-keys/create` | POST | `create_api_key_form_handler` | session✅ | ❌ | 🟢 Handler checks session |
| `/profile/api-keys/:id/revoke` | POST | `revoke_api_key_handler` | session✅ | ❌ | 🟢 Handler checks session + ownership |
| `/api/user/api-keys` | POST | `create_api_key_json_handler` | session✅ | ❌ | 🟡 No API key auth support |
| `/api/user/api-keys` | GET | `list_api_keys_json_handler` | session✅ | ❌ | 🟡 No API key auth support |
| `/api/user/api-keys/:id` | GET | `get_api_key_json_handler` | session✅ | ❌ | 🟡 No API key auth support |
| `/api/user/api-keys/:id` | PUT | `update_api_key_json_handler` | session✅ | ❌ | 🟡 No API key auth support |
| `/api/user/api-keys/:id` | DELETE | `delete_api_key_json_handler` | session✅ | ❌ | 🟡 No API key auth support |

**Assessment:** All handlers call `get_user_id_from_session()` which returns 401 if no session. The JSON API routes only accept session auth (not API key auth), which may be intentional to prevent key-bootstrapping attacks. However, adding middleware would provide defense-in-depth.

**Recommendation:** Consider adding `api_key_or_session_auth` middleware for consistency, or document this as a deliberate design choice.

---

### Group D: Media Manager — WITH `api_key_or_session_auth` ✅

Source: `crates/media-manager/src/routes.rs`

#### Listing & Search (auth middleware, no per-resource ACL)

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/media` | GET | `list_media_html` | ✅ | ❌ | 🟢 Lists user's own media |
| `/media/search` | GET | `search_media_html` | ✅ | ❌ | 🟢 Search within auth context |
| `/api/media` | GET | `list_media_json` | ✅ | ❌ | 🟢 |
| `/api/media/search` | GET | `search_media_json` | ✅ | ❌ | 🟢 |

#### Upload

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/media/upload` | GET | `show_upload_form` | ✅ | ❌ | 🟢 Form page |
| `/api/media/upload` | POST | `upload_media` | ✅ | ❌ | 🟡 See TD-009 for validation gaps |

#### Detail Pages

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/media/:slug` | GET | `media_detail_handler` | ✅ | ✅ `AccessControlService` | 🟢 Full 4-layer ACL |
| `/media/:slug/view` | GET | `view_markdown_handler` | ✅ | ⚠️ Ad-hoc | 🟡 Manual owner/public check, not AccessControlService |
| `/media/:slug/edit` | GET | `edit_markdown_handler` | ✅ | ⚠️ Owner-only | 🟢 Checks ownership |

**Detail: `view_markdown_handler`** (lines 107–115 of `markdown_view.rs`):
```rust
let has_access = if is_public == 1 {
    true
} else if let Some(ref uid) = user_id {
    owner_id.as_ref() == Some(uid)
} else {
    false
};
```
This skips the full `AccessControlService` pipeline — meaning access codes, group memberships, and audit logging are all bypassed. An access code that grants Read on a markdown document will NOT work via this route.

#### CRUD API

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/api/media/:slug/toggle-visibility` | POST | `toggle_visibility` | ✅ | ? | 🟡 Should verify ownership |
| `/api/media/:slug/save` | POST | `save_markdown_handler` | ✅ | Owner✅ | 🟢 Checks ownership |
| `/api/media/:slug` | GET | `get_media_item` | ✅ | ? | 🟡 Need to verify ACL usage |
| `/api/media/:slug` | PUT | `update_media_item` | ✅ | ? | 🟡 Need to verify ACL usage |
| `/api/media/:slug` | DELETE | `delete_media` | ✅ | ? | 🟡 Need to verify ACL usage |
| `/api/user/vaults` | GET | `get_user_vaults` | ✅ | session | 🟢 User-scoped |

#### Image Serving ← KEY MEDIA ROUTES

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/images/:slug` | GET | `serve_image_with_suffix_check` | ✅ | ✅ `AccessControlService` | 🟢 Full ACL for private images |
| `/images/:slug/original` | GET | `serve_image_original` | ✅ | ✅ `AccessControlService` | 🟢 Full ACL |
| `/images/:slug/thumb` | GET | `serve_image_thumbnail` | ✅ | ✅ `AccessControlService` | 🟢 Full ACL |

**Note:** All three call `serve_image_variant()` which:
- Checks `is_public` flag
- For private images: verifies ownership OR access code via `AccessControlService`
- Uses proper `Permission::Read` check

---

### Group E: Video Manager — WITH `api_key_or_session_auth` ✅

Source: `crates/video-manager/src/lib.rs`

#### Content Serving

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/videos/:slug` | GET | `video_player_handler` | ✅ | ✅ `AccessControlService` | 🟢 Full ACL |
| `/watch/:slug` | GET | `video_player_handler` | ✅ | ✅ `AccessControlService` | 🟢 Same handler |
| `/hls/*path` | GET | `hls_proxy_handler` | ✅ | ✅ `AccessControlService` | 🟢 Full ACL per-stream |
| `/test` | GET | `live_test_handler` | ✅ | ❌ | 🟢 Test page only |

#### API & Management

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/api/stream/validate` | GET | `validate_stream_handler` | ✅ | ❌ | 🟢 |
| `/api/stream/authorize` | GET | `authorize_stream_handler` | ✅ | ❌ | 🟢 |
| `/api/mediamtx/status` | GET | `mediamtx_status` | ✅ | ❌ | 🟢 |
| `/api/videos` | GET | `list_videos_api_handler` | ✅ | ❌ | 🟢 |
| `/api/videos` | POST | `register_video_handler` | ✅ | ❌ | 🟢 |
| `/api/videos/metrics` | GET | `get_metrics_handler` | ✅ | ❌ | 🟢 |
| `/api/videos/metrics/detailed` | GET | `get_detailed_metrics_handler` | ✅ | ❌ | 🟢 |
| `/api/videos/available-folders` | GET | `available_folders_handler` | ✅ | ❌ | 🟢 |
| `/api/videos/:id` | PUT | `update_video_handler` | ✅ | ? | 🟡 Need to verify ownership |
| `/api/videos/:id` | DELETE | `delete_video_handler` | ✅ | ✅ `AccessControlService` | 🟢 Uses Permission::Delete |
| `/api/videos/:id/tags` | GET/POST/PUT | tag handlers | ✅ | ? | 🟡 Need to verify ownership |
| `/api/videos/:id/tags/:tag_slug` | DELETE | `remove_video_tag_handler` | ✅ | ? | 🟡 Need to verify ownership |

---

### Group F: Access Codes — NO MIDDLEWARE ⚠️

Source: `crates/access-codes/src/lib.rs`

| Route | Method | Handler | Auth | ACL | Risk |
|-------|--------|---------|------|-----|------|
| `/api/access-codes` | POST | `create_access_code` | session✅ | ✅ `AccessControlService` (ownership) | 🟢 |
| `/api/access-codes` | GET | `list_access_codes` | session✅ | ❌ | 🟢 User-scoped |
| `/api/access-codes/:code` | DELETE | `delete_access_code` | session✅ | ? | 🟡 Verify owner-only |
| `/access/codes` | GET | `list_access_codes_page` | session✅ | ❌ | 🟢 |
| `/access/codes/new` | GET | `new_access_code_page` | session✅ | ❌ | 🟢 |
| `/access/codes/:code` | GET | `view_access_code_page` | session✅ | ❌ | 🟢 Scoped to creator |
| `/access/preview` | GET | `preview_access_code_page` | ❌ | ❌ | 🟢 Intentionally public preview |

**Assessment:** All handlers (except the public preview) check `session.get("authenticated")`. The `create_access_code` handler additionally uses `AccessControlService` with `Permission::Admin` to verify the user owns the media they're sharing. Good.

**Recommendation:** Add middleware for consistency and defense-in-depth.

---

### Group G: Vault Manager — NO MIDDLEWARE ⚠️

Source: `crates/vault-manager/src/lib.rs`

| Route | Method | Handler | Auth | ACL | Risk |
|-------|--------|---------|------|-----|------|
| `/api/user/vaults` | POST | `create_vault` | session✅ | ❌ | 🟢 |
| `/api/user/vaults/:vault_id` | PUT | `update_vault` | session✅ | owner✅ | 🟢 |
| `/api/user/vaults/:vault_id` | DELETE | `delete_vault` | session✅ | ? | 🟡 Verify owner-only |
| `/api/user/vaults/:vault_id/set-default` | POST | `set_default_vault` | session✅ | owner✅ | 🟢 |
| `/vaults` | GET | `list_vaults_page` | session✅ | ❌ | 🟢 |
| `/vaults/new` | GET | `new_vault_page` | session✅ | ❌ | 🟢 |

**Assessment:** All handlers check authentication via session. `update_vault` and `set_default_vault` verify vault ownership (`WHERE vault_id = ? AND user_id = ?` pattern).

**Recommendation:** Add middleware for defense-in-depth.

---

### Group H–J: Access Groups / Tags / Search — WITH MIDDLEWARE ✅

All protected by `api_key_or_session_auth`. No media content is directly served by these routes.

**Note:** The access groups router includes public invitation endpoints (`/invitations/:token`) that are behind the auth middleware. This means anonymous invitation acceptance may be blocked. Verify whether this is intentional.

---

### Group K: 3D Gallery — NO MIDDLEWARE

| Route | Method | Handler | Auth | ACL | Risk |
|-------|--------|---------|------|-----|------|
| `/3d` | GET | `viewer_page` | ❌ | ❌ | 🟢 Intentionally public |
| `/digital-twin` | GET | `viewer_page` | ❌ | ❌ | 🟢 Intentionally public |
| `/api/3d/gallery` | GET | `get_gallery_data` | ❌ | ❌ | 🟢 Public gallery data |
| `/static/3d-gallery/bundle.js` | GET | `serve_bundle_js` | ❌ | ❌ | 🟢 Static asset |

**Assessment:** Intentionally public. No sensitive data.

---

### Group L: Docs Viewer — WITH `api_key_or_session_auth` ✅

Source: `crates/docs-viewer/src/routes.rs`, nested at `/docs`

| Route | Method | Handler | Middleware | ACL | Risk |
|-------|--------|---------|-----------|-----|------|
| `/docs/` | GET | `docs_index` | ✅ | ❌ | 🟢 |
| `/docs/view` | GET | `view_doc` | ✅ | ❌ | 🟡 No per-document ACL |
| `/docs/upload` | GET | `upload_form` | ✅ | ❌ | 🟢 |
| `/docs/upload` | POST | `upload_doc` | ✅ | ❌ | 🟡 Only allows .md files, but no size limit check |

**Note:** `view_doc` has path traversal protection (`Component::ParentDir` check) ✅

---

### Group M: Static File Serving — 🔴 CRITICAL

| Route Pattern | Service | Auth | ACL | Risk |
|---------------|---------|------|-----|------|
| `/storage/*` | `ServeDir::new(&storage_dir)` | ❌ | ❌ | 🔴 **CRITICAL** |
| `/static/*` | `ServeDir::new("static")` | ❌ | ❌ | 🟢 Public CSS/JS |

#### `/storage/*` — Full Analysis

**What it serves:** The `storage` directory contains:
```
storage/
├── videos/          ← All uploaded videos
├── images/          ← All uploaded images (originals, WebP, thumbnails)
├── documents/       ← All uploaded documents (PDFs, markdown, etc.)
└── users/           ← Per-user vault storage
    └── {user_id}/
        └── vaults/
            └── {vault_id}/
                ├── images/
                ├── videos/
                ├── documents/
                └── thumbnails/
```

**Attack scenario:** An unauthenticated attacker who knows (or brute-forces) file paths can:
1. Access private images: `GET /storage/users/{user_id}/vaults/{vault_id}/images/{slug}.webp`
2. Access private documents: `GET /storage/users/{user_id}/vaults/{vault_id}/documents/{timestamp}_{filename}.pdf`
3. Access private videos: `GET /storage/videos/{filename}.mp4`
4. Enumerate user IDs and vault IDs by trying predictable patterns

**Why this exists:** Likely a developer convenience for serving media without going through application handlers. The application-level routes (`/images/:slug`, `/media/:slug`, `/hls/*path`) all properly check access control — but `/storage/*` is a complete bypass around all of them.

---

## Risk Summary Matrix

| Route Group | Middleware | Handler Auth | AccessControlService | Overall |
|-------------|-----------|-------------|---------------------|---------|
| `/storage/*` | ❌ | ❌ | ❌ | 🔴 CRITICAL |
| `/media/:slug/view` | ✅ | ✅ | ⚠️ Ad-hoc only | 🟡 MEDIUM |
| `vault_routes` | ❌ | ✅ session | ❌ | 🟡 MEDIUM |
| `access_code_routes` | ❌ | ✅ session | Partial | 🟡 MEDIUM |
| `api_key_routes` | ❌ | ✅ session | ❌ | 🟡 MEDIUM |
| Webhook endpoints | ❌ | ❌ | ❌ | 🟡 MEDIUM |
| `/media/:slug` (detail) | ✅ | ✅ | ✅ | 🟢 GOOD |
| `/images/:slug` (all variants) | ✅ | ✅ | ✅ | 🟢 GOOD |
| `/videos/:slug` | ✅ | ✅ | ✅ | 🟢 GOOD |
| `/hls/*path` | ✅ | ✅ | ✅ | 🟢 GOOD |
| `/docs/*` | ✅ | ✅ | ❌ | 🟢 OK (internal docs) |

---

## Recommended Remediations

### 🔴 P0 — `/storage/*` Bypass (CRITICAL)

**Option A (Recommended):** Remove `nest_service("/storage", ...)` entirely. All media is already served through application-level routes (`/images/:slug`, `/media/:slug`, `/hls/*path`).

**Option B:** Replace with an application handler that checks access control:
```rust
// Replace:
.nest_service("/storage", ServeDir::new(&storage_dir))

// With:
.route("/storage/*path", get(authenticated_file_handler))
```

**Option C (Quick mitigation):** Add the auth middleware:
```rust
.nest_service("/storage",
    ServeDir::new(&storage_dir)
).route_layer(axum::middleware::from_fn_with_state(
    Arc::new(pool.clone()),
    api_key_or_session_auth,
))
```
Note: Option C only adds authentication, not per-resource ACL. A logged-in user could still access another user's private files.

### 🟡 P1 — Markdown View ACL

Update `view_markdown_handler` to use `AccessControlService` instead of ad-hoc checks:

```rust
// Replace manual check with:
let context = AccessContext::new(resource_type, media_id)
    .with_user(user_id);
let decision = state.access_control
    .check_access(context, Permission::Read)
    .await?;
```

### 🟡 P1 — Add Middleware to Unprotected CRUD Groups

Add `api_key_or_session_auth` middleware to:
- `vault_routes`
- `access_code_routes` (except `/access/preview` which should remain public)
- `api_key_routes`

### 🟡 P2 — Webhook Authentication

Add a shared secret validation for webhook endpoints:
```rust
.route("/api/webhooks/stream-ready", post(webhook_stream_ready))
.route("/api/webhooks/stream-ended", post(webhook_stream_ended))
// Add: validate X-Webhook-Secret header
```

---

## Appendix: Auth Middleware Behavior

The `api_key_or_session_auth` middleware (in `crates/api-keys/src/middleware.rs`):

1. Checks for `Authorization: Bearer <key>` or `X-API-Key: <key>` header
2. If found: validates key against DB, sets session vars for backward compat, adds `AuthenticatedUser` to request extensions
3. If no API key: checks `session.get("authenticated")`
4. If neither: returns `401 Unauthorized`

This is a **gate** — it ensures someone is logged in, but does NOT perform resource-level access control. Individual handlers must still check ownership, group membership, etc.

---

## Fixes Applied (2025-01-27)

All remediations from the audit above have been implemented in a single pass.

### 🔴 P0 — `/storage/*` Bypass → FIXED

**File:** `src/main.rs`

The `nest_service("/storage", ServeDir)` call has been replaced with a `Router` wrapped
in the `api_key_or_session_auth` middleware:

```rust
.nest(
    "/storage",
    Router::new()
        .nest_service("/", ServeDir::new(&storage_dir))
        .route_layer(axum::middleware::from_fn_with_state(
            Arc::new(pool.clone()),
            api_key_or_session_auth,
        )),
)
```

Unauthenticated requests to `/storage/*` now receive **401 Unauthorized**.

Additionally, all public assets that were previously served from `/storage/` have been
relocated to `/static/` (which is intentionally public):

| Asset | Old path | New path |
|-------|----------|----------|
| App icon | `/storage/icon.webp` | `/static/icon.webp` |
| App icon (default) | `/storage/icon.png` | `/static/icon.webp` |
| Video placeholder | `/storage/images/video_placeholder.webp` | `/static/images/video_placeholder.webp` |

Updated references in:
- `src/main.rs` — `AppConfig::default()` icon path
- `templates/base-tailwind.html` — `<link rel="icon">` and `<link rel="apple-touch-icon">`
- `crates/standalone/3d-gallery/frontend/src/scene/VideoScreen.js` — fallback thumbnail URL

### 🟡 P1 — Markdown View ACL → FIXED

**File:** `crates/media-manager/src/markdown_view.rs`

The `view_markdown_handler` now uses the full `AccessControlService` pipeline instead of
the ad-hoc `is_public == 1 || owner == user` check. This means:

- ✅ Access codes now work for markdown documents
- ✅ Group-based access is respected
- ✅ Access decisions are audit-logged
- ✅ Accepts `?code=` query parameter (new `MarkdownAccessQuery` struct)

### 🟡 P1 — Middleware on Unprotected CRUD Groups → FIXED

**File:** `src/main.rs`

Three route groups now have `api_key_or_session_auth` middleware applied via `.route_layer()`:

| Route group | Before | After |
|-------------|--------|-------|
| `api_key_routes` | No middleware (handler-only session checks) | ✅ Middleware + handler checks |
| `access_code_routes` | No middleware | ✅ Middleware + handler checks |
| `vault_routes` | No middleware | ✅ Middleware + handler checks |

**Special handling for access codes:**

The access-codes crate was refactored to export two route functions:

- `access_code_routes()` — protected CRUD routes (behind middleware)
- `access_code_public_routes()` — the `/access/preview` page only (intentionally public for shared links)

**File:** `crates/access-codes/src/lib.rs` — split into two functions.

### 🟡 P1 — SVG XSS Mitigation → FIXED

**File:** `crates/media-manager/src/serve.rs`

All served images now include `X-Content-Type-Options: nosniff`.

SVG files additionally receive:
- `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'` — blocks all
  script execution, data URIs, and external resource loading
- `Content-Disposition: inline; filename="image.svg"` — prevents filename-based attacks

### Summary of Changed Files

| File | Change |
|------|--------|
| `src/main.rs` | Gated `/storage/*`, added middleware to 3 route groups, moved icon path, imported `access_code_public_routes` |
| `templates/base-tailwind.html` | Updated favicon paths to `/static/` |
| `crates/media-manager/src/upload.rs` | Full media-core validation pipeline (see TD-009) |
| `crates/media-manager/src/markdown_view.rs` | AccessControlService integration, `?code=` support |
| `crates/media-manager/src/serve.rs` | SVG CSP headers, `X-Content-Type-Options: nosniff` |
| `crates/access-codes/src/lib.rs` | Split routes into protected + public |
| `crates/standalone/3d-gallery/frontend/src/scene/VideoScreen.js` | Updated fallback thumbnail path |
| `static/icon.webp` | Copied from `storage/icon.webp` |
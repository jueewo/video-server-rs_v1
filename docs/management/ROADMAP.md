# Platform Roadmap

> Guiding principle: **maximum consolidation, no bloating of concepts.**
> One place files live. One way to upload. One way to share.
> External tools can always get data from the platform.

---

## Tagline

> **"Run your business from one place. On your own server."**

## Vision

The operating system for a small business. Workspaces are the company, folders are
departments or projects, apps are the tools those departments use.

The platform stores, transcodes, streams, models, trains, and exposes content.
Some apps are built-in (BPMN, 3D space, courses, media). Others connect via open
interfaces (WebDAV, API, MCP) — including custom Vue3/Preact data platforms built
as consulting work product and delivered via js-tool folder types.

---

## Architecture Foundation — Universal Shell + Dual-Use Crates

**Goal:** Establish the patterns everything else is built on. No user-visible changes.

### The Universal Shell

The workspace browser is a thin frame. It has no knowledge of specific folder types
at compile time. Each folder type registers a `FolderTypeRenderer` — a trait defined
in `workspace-core` — and provides an inline HTML view. The shell renders the frame
(breadcrumbs, nav, workspace name); the renderer owns the content area.

```
workspace-core/          ← FolderTypeRenderer trait, FolderViewContext
workspace-manager/       ← shell only: routing, config, delegates to renderers
main.rs                  ← wires renderers: media, bpmn, course, docs, ...
```

New folder type = new crate + one line in `main.rs`. `workspace-manager` never changes.

### Dual-Use Crates

Every functional crate is a library that can run in two modes:

| Mode | How | Use case |
|---|---|---|
| Embedded | Implements `FolderTypeRenderer` | Inline view in workspace browser |
| Standalone | Exposes its own `Router` + minimal shell | Independent deployable app |

The standalone binary in `crates/standalone/` is 10–20 lines — loads config,
calls `crate::standalone::run()`. Core logic, templates, and state are written
once and reused in both modes. No duplication.

**Examples of dual-use:**
- `crates/bpmn/` → workspace: diagram list inline | standalone: process simulator
- `crates/media/` → workspace: media grid inline  | standalone: media server
- `crates/course/` → workspace: lesson outline    | standalone: course platform

Any block can be extracted and delivered as a focused standalone tool for a client
without changing the core codebase.

### Tasks

- [x] Define `FolderTypeRenderer` trait in `workspace-core` crate
- [x] Refactor `workspace-manager` to accept `Vec<Arc<dyn FolderTypeRenderer>>`
- [x] Migrate `bpmn-viewer` as first proof-of-concept (smallest, most self-contained)
      - Enhanced: recursive subfolder support (any depth), grouped by relative subfolder path, client-side search filter (2026-03-07)
- [x] Migrate `media-manager` folder view (replaces Phase 0.5 redirect)
- [x] Workspace dashboard enriched: total size, file-type composition chips, folder type badges,
      clickable filenames, inline image thumbnails in Recent Files (2026-03-07)
- [x] PDF viewer defaults to fit-to-width (2026-03-07)
- [x] **`crates/course/` — first full dual-use crate** (2026-03-08)
      - Embedded: `CourseFolderRenderer` — module/lesson outline in workspace browser
      - Standalone: `GET /course?code={code}` — full course viewer with own shell, no session
      - `course.yaml` optional for title/ordering overrides; pure filesystem inference otherwise
      - Markdown rendering with asset URL rewriting (images served via `?code=` without session)
      - Old `crates/standalone/course-viewer/` removed (was unimplemented, vault-model)
      - See `docs/apps/course-viewer.md`
- [x] `crates/media-viewer/` dual-use — media viewer/gallery (2026-03-09)
      - Embedded: `MediaViewerRenderer` replaces `MediaFolderRenderer` from media-manager
      - Standalone: `GET /gallery?code={code}` — public gallery, no session
      - See `docs/apps/media-viewer.md`
- [x] Document the pattern for adding new folder types → `docs/apps/DUAL_USE_PATTERN.md` (2026-03-09)

---

## Phase 0.5 — Media-Server Folder Type (Bridge Step)

**Goal:** Unify workspace UX with the media pipeline without a storage migration.

**Implemented 2026-03-06** — see `docs/management/media-server-folder-type.md`

- [x] Register `media-server` folder type in the folder-type registry (film icon, #6366f1, builtin YAML)
- [x] Auto-create a vault when a folder is assigned `folder-type: media-server`
- [x] Store `vault_id` in `workspace.yaml` folder metadata
- [x] Redirect to `/media?vault_id=...` when a media-server folder is opened
- [x] `GET /api/workspaces/{id}/media-folders` — lists media-server folders with vault_id for the picker
- [x] Replace "Publish to Vault" modal with compact "→ Media" popover: auto-sends if one media-server
      folder exists, shows picker if multiple, guides user to create one if none
- [x] `publish_to_vault` uses `vault_nested_media_dir` (correct nested path), detects `MediaType`
      from MIME, auto-infers title from filename stem
- [x] Restrict → Media button and server-side handler to `image/*`, `video/*`, `application/pdf` only
      — text/yaml/markdown stay as plain workspace files
- [x] Access-code step removed from publish flow (handled separately per plan)

**Result:** Users can create a "Media Server" folder in any workspace. Opening it takes
them directly to the scoped media manager. Each media-server folder is backed by an
isolated vault. Publishing a file from the workspace browser to media is a single click.
No storage migration required.

---

## Phase 1 — Complete the Access Model

**Goal:** Close the access model. Internal users reach media through workspace folders
(done). External clients and satellite apps need a parallel path — folder-scoped access
codes — without requiring user accounts.

**Storage does not move.** Vault paths stay as-is; vault_id is already an internal
implementation detail hidden behind the media-server folder type. The heavy storage
consolidation (path unification, `media_items` as index) belongs in Phase 2 when
transcoding becomes a service. Phase 1 is about access, not storage.

### Access model completed

| Who | Path | Status |
|---|---|---|
| Internal user | Workspace → media-server folder → inline grid | ✓ Done |
| External client / satellite app | Folder-scoped access code → media list + serving | ✓ Done |

### Tasks

- [x] **Workspace access codes** — a code that unlocks a workspace folder (media-server
      folders map to vault_id internally; BPMN/docs/course folders serve raw files).
      Satellite apps present the code to get media lists + serving URLs. No user account
      needed. Internal users can claim codes to see shared content. Multiple folders per
      code supported. Optional group_id scoping for media-server folders. (2026-03-07)
- [x] **Per-item codes remain** for sharing a single file with a client (existing feature,
      kept as-is)
- [x] **API endpoints for folder codes**:
      - `GET /api/folder/{code}/media` — media items with serving URLs (no session)
      - `GET /api/folder/{code}/files` — raw files with serve URLs (no session), recursive
      - `PATCH /api/workspace-access-codes/{code}` — update description / expiry
      - `POST /api/workspace-access-codes/{code}/folders` — add folder to existing code
      - All `/media/{slug}/...` and `/api/workspaces/{id}/files/serve` routes accept `?code=` (2026-03-07)
- [x] **Management UI enhancements** — folder path badges per code; inline description
      editing; "Add to existing" tab in Share modal with active-code dropdown (2026-03-07)
- [ ] **Hide `/media` standalone entry points** — `/media` global list and vault picker
      are internal scaffolding; users should reach media only through workspace folders.
      Deprecate or gate behind auth without workspace context.

**Result:** Two clean access paths. Internal = workspace browser. External = access code.
No vault concept visible to any user or satellite app.

**Result:** Users have one mental model. Files live in workspaces. Period.

---

## Phase 2 — Transcoding as a Service

**Goal:** HLS transcoding and thumbnail generation are jobs applied to workspace files,
not reasons to store files differently.

- [ ] Trigger transcoding on a workspace file (video) — output written back to workspace
- [ ] Thumbnail generation on demand for workspace files
- [ ] WebP conversion for images as a serving-time transform, not a stored copy
- [ ] Progress tracking (WebSocket) tied to workspace file path, not media slug
- [ ] RTMP live streaming remains independent (MediaMTX integration unchanged)

**Result:** The transcoding pipeline is a service. Storage is unchanged.

---

## Phase 3 — Open Access Layer

**Goal:** External tools can always get data from the platform via open interfaces.

- [ ] Stable public API surface documented — media serving routes, file serving, metadata
- [ ] Workspace-level access codes usable by external apps (no user account needed)
- [ ] API keys for programmatic / satellite app access
- [ ] WebDAV stable and documented for filesystem-level consumers
- [ ] MCP server updated to reflect workspace-first model

**Result:** Satellite apps (course platform, 3D gallery, etc.) connect cleanly
without being part of this codebase.

---

## Phase 4 — Satellite Apps & Integration

**Goal:** Clean separation between core business features and infrastructure satellites.

Core business features (3D space, course viewer, BPMN) stay in the platform —
they are part of the SMB story. Infrastructure satellites (tools that consume the
platform as a content backend) connect via open interfaces.

- [ ] Update folder-type app links to support external URLs (not just internal routes)
- [ ] Document the integration pattern for satellite apps
- [ ] Vue3/Preact data platforms (e.g. pharma industry prototypes) deployable as
      js-tool folder types — consulting work product delivered via the platform
- [x] `crates/course/` migrated to dual-use (2026-03-08) — see above
- [ ] `crates/standalone/3d-gallery` — evaluate for dual-use migration
- [ ] `crates/media/` dual-use — standalone media viewer/gallery consuming `/api/folder/{code}/media`

**Result:** Platform is the delivery vehicle for consulting and business work product.

---

## Phase 5 — App Ecosystem

**Goal:** The folder-type + app registry becomes a real integration layer.

- [ ] App registry as a proper YAML registry (installable, not just builtin types)
- [ ] External app URLs in app links (folder type points to external satellite app)
- [ ] Public access to folder-typed content via access codes
- [ ] Admin installs/disables apps from the /apps page

**Result:** Third parties can publish apps that integrate with the platform.
The workspace becomes a platform others build on.

---

## Phase 6 — Multi-Tier Delivery

**Goal:** Package and deliver the platform to three distinct customer types from one
codebase. No new features — get the core delivery model right before adding modules.

See `docs/management/DELIVERY_TIERS.md` for the full design.

```
Tier 1 — Your hosted platform     (B2C + your own use)
Tier 2 — Hosted B2B               (company on your infrastructure, tenant-scoped)
Tier 3 — Standalone               (company on their own infrastructure, single-tenant)
```

### Phase 6A — Standalone packaging (Tier 3) ~1 week

- [x] `deployment_mode` field in config — `standalone` locks to single tenant at startup (2026-03-09)
- [x] Rename `app.yaml` → `branding.yaml`, extend with logo, colors, support email (2026-03-09)
- [x] Cargo `[features]`: `media`, `course`, `bpmn`, `full` — compile only what's licensed (2026-03-09)
- [x] Conditional `register_renderer()` and `.merge(routes)` in `main.rs` per feature (2026-03-09)
- [x] `Dockerfile` — binary + FFmpeg + Ghostscript + cwebp; `FEATURES` build arg (2026-03-09)
- [x] `docker-compose.standalone.yml` — storage volume, DB, config mounts for customer self-hosting (2026-03-09)
- [x] `docs/deployment/STANDALONE_CONFIG.md` — self-hosting configuration guide (2026-03-09)

**Result:** A standalone customer (e.g. regulated industry) receives a Docker image
with exactly their licensed features, their branding, pointing at their own IdP.
Single-tenant enforced. No data leaves their infrastructure.

### Phase 6B — Tenant scoping (Tier 2) ~1 week

- [x] `tenants` table — one row per company + `'platform'` row for your workspaces (2026-03-09)
- [x] `tenant_id` column on `workspaces` + `users` — migrate existing rows to `'platform'` (2026-03-09)
- [x] Session resolves `tenant_id` after login (OIDC + emergency paths) (2026-03-09)
- [x] Workspace list query scoped to `WHERE tenant_id = ?` (2026-03-09)
- [x] Workspace create stores `tenant_id` from session (2026-03-09)
- [x] Per-tenant branding stored as JSON on tenant row (schema in place) (2026-03-09)
- [x] Minimal tenant admin API — `POST /api/admin/tenants`, `GET /api/admin/tenants`, `PUT /api/admin/users/{id}/tenant` (2026-03-09)
- [ ] Tenant admin UI page — provision tenants, assign users (Phase 6C)
- [ ] Per-tenant branding resolved per session and applied to templates (Phase 6C)
- [ ] Hosted B2B onboarding flow — create tenant → workspace → invite users (Phase 6C)

**Result:** A company can be onboarded as a tenant on your platform. Their users
see only their workspaces. White-label branding per tenant. Their end users consume
content via access codes as before.

---

## What Stays Out of Scope

- PostgreSQL support (SQLite is sufficient for the target use case)
- Real-time collaboration (single-user editing is acceptable for now)
- Mobile apps (web-first)
- Built-in course authoring, 3D scene editing — those belong in satellite apps

---

## Non-Negotiables

- Self-hosted, single binary
- Data sovereignty — no external service dependencies for core functionality
- Sharing must work without requiring recipient user accounts (access codes)
- External tools must be able to read content (WebDAV, API, MCP)

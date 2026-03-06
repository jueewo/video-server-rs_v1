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
- [x] Migrate `media-manager` folder view (replaces Phase 0.5 redirect)
- [ ] Document the pattern for adding new folder types

---

## Phase 0.5 — Media-Server Folder Type (Bridge Step)

**Goal:** Unify workspace UX with the media pipeline without a storage migration.

**Implemented 2026-03-06** — see `docs/management/media-server-folder-type.md`

- [x] Register `media-server` folder type in the folder-type registry
- [x] Auto-create a vault when a folder is assigned `folder-type: media-server`
- [x] Store `vault_id` in `workspace.yaml` folder metadata
- [x] Redirect to `/media?vault_id=...` when a media-server folder is opened

**Result:** Users can create a "Media Server" folder in any workspace. Opening it takes
them directly to the scoped media manager. Each media-server folder is backed by an
isolated vault. No storage migration required.

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
| External client / satellite app | Folder-scoped access code → media list + serving | Phase 1 |

### Tasks

- [ ] **Folder-scoped access codes** — a code that unlocks all media in a workspace
      folder (maps to vault_id internally). Satellite apps (3D gallery, course viewer)
      present the code to get a media list and serving URLs. No user account needed.
- [ ] **Per-item codes remain** for sharing a single file with a client (existing feature,
      kept as-is)
- [ ] **API endpoint for folder code** — `GET /api/folder/{code}/media` returns
      accessible items + serving URLs for the satellite app to consume
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
- [ ] Evaluate `crates/standalone/3d-gallery` and `crates/standalone/course-viewer`:
      migrate to dual-use library crates (embedded + standalone) per the Architecture
      Foundation pattern, or keep as standalone-only if they never embed in the workspace

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

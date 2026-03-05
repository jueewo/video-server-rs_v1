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

## Phase 1 — Consolidate Storage (Foundation)

**Goal:** One storage model. Workspaces are the authoritative home for all files.

- [ ] Retire vault as a user-facing concept. Vault storage becomes an internal implementation
      detail or is merged into workspace storage
- [ ] `media_items` table becomes a lightweight index over workspace files (path, mime, metadata)
      rather than the authoritative record with its own storage location
- [ ] Upload flow goes to workspace, not vault. No more "Publish to Vault" step
- [ ] Serving routes read from workspace storage
- [ ] Remove the dual-path fallback logic (`find_media_file`, `vault_nested_media_dir` complexity)
- [ ] Define workspace-level access codes for sharing individual files or folders

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
- [ ] Evaluate whether `crates/standalone/3d-gallery` and `crates/standalone/course-viewer`
      benefit from extraction, or are better hardened in-place as core features

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

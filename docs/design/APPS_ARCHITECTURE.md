# Apps Architecture

## Concept

The platform has three tiers:

| Tier | Purpose | Who uses it |
|---|---|---|
| **Media collection** | Assets — videos, images, documents, with access control | Owner + shared users |
| **Workspace** | Workshop — files created, edited, configured; internal use or setup for automated processes | Owner only |
| **Apps** | Published results — user-facing websites, webapps, courses, galleries | End users / students |

Apps are the **output layer**. The workspace is where you **build and prepare**. Media is the **asset store**.

---

## App Lifecycle

```
Workspace folder (typed)            App Template (optional)
  ├── lesson-01.md              +     ├── src/layouts/
  ├── lesson-02.md                    ├── astro.config.mjs
  └── meta.yaml                       └── package.json
          │                                   │
          └──────── build step ───────────────┘
                         │
                    (or just copy
                    for js-tools)
                         │
                         ▼
              storage-apps/{slug}/       ← static output snapshot
                  ├── index.html
                  └── ...
                         │
                         ▼
            /pub/{slug}  ──►  end users access
```

**Key principle:** workspace stays clean — only the actual content lives there, no framework boilerplate. The app template provides the structure and UI. Build merges them.

---

## Build Strategies (per folder type)

| Folder type | Build strategy | Template |
|---|---|---|
| `js-tool` | Copy only — files are already static | None needed |
| `course` | Astro / Vitepress build — markdown → HTML | `app-templates/course-astro/` |
| `bpmn-simulator` | Copy + generate index wrapping BPMN viewer | Minimal HTML wrapper |
| *(future)* `3d-scene` | Copy + generate gallery index | Minimal HTML wrapper |

Build runs async (like HLS transcoding): spawns a process, tracks progress, output lands in `storage-apps/{slug}/`.

---

## App Complexity Levels (js-tools)

| Level | Tech | Persistence | Works today? |
|---|---|---|---|
| 1 — Static | Plain HTML/JS | None | ✅ |
| 2 — SPA | Vue3 / Preact (pre-built) | None | ✅ |
| 3 — Client DB | Vue3 + IndexedDB / SQLite-WASM | Browser-local | ✅ |
| 4 — Shared backend | Any + server API | Shared DB | ❌ Requires own process (out of scope) |

---

## Access Control

Published apps reuse the existing access-code infrastructure:

- **Public** — no auth required, anyone with the URL can access
- **Code** — shareable link with a code (same pattern as media items)
- **Private** — owner-only (default until explicitly published)

---

## Data Model

> **Note:** The `published_apps` table has been superseded by the unified `publications` table. See [Publications Architecture](PUBLICATIONS_ARCHITECTURE.md) for the current schema. Existing `published_apps` rows were migrated into `publications` with `slug = app_id`.

Published output stored at: `storage-apps/{slug}/` (snapshot, separate from workspace).

---

## URL Design

> **Note:** App publishing is now handled by the unified publications system. See [Publications Architecture](PUBLICATIONS_ARCHITECTURE.md) for the current route table.

| URL | Purpose |
|---|---|
| `/pub/{slug}` | Public entry point for a published app |
| `/pub/{slug}/{*path}` | File serving within the app |
| `/pub/{slug}?code=XXX` | Access-code protected entry |
| `/api/publications` | POST: publish a workspace folder; GET: list publications |
| `/api/publications/{slug}` | PUT / DELETE: manage a publication |
| `/my-publications` | Owner's dashboard of all publications |

> `/apps` (platform apps page showing 3D gallery, Course Viewer, etc.) is distinct from `/pub/` (user-published content).

---

## Implementation Progress

### Phase 1 — Foundation ✅
- [x] js-tool viewer crate — serves static files from workspace folders (auth-gated, owner-only)
- [x] AppLink on folder type definitions — connects folder types to app URLs
- [x] "Open App" button in workspace folder detail view
- [x] Folder gallery at `/js-apps/{workspace_id}/{folder}` (owner preview)
- [x] `publications` DB table (migrated from `published_apps`) — unified registry
- [x] Publish API: `POST /api/publications` — copy files to `storage-apps/{slug}/`
- [x] Public serving: `GET /pub/{slug}` + `GET /pub/{slug}/{*path}` with access control
- [x] Access control: public + code support on `/pub/` routes (publications crate)
- [x] "Publish / Launch" button in workspace folder detail view (typed folders only)

### Phase 2 — Management ✅
- [x] List API: `GET /api/publications` — user's publications
- [x] Edit API: `PUT /api/publications/{slug}` — title, description, access mode
- [x] Delete API: `DELETE /api/publications/{slug}` — removes snapshot + DB record
- [x] Re-publish: `POST /api/publications/{slug}/republish` — copy workspace again
- [x] `/my-publications` dashboard — all publications with status, URLs, access settings
- [x] `/catalog` — public catalog of all public publications

### Phase 3 — Build Pipeline
- [ ] App templates registry (`storage/app-templates/` or embedded)
- [ ] Build step support — async process like HLS transcoding
- [ ] Course folder type → Astro build → published course site
- [ ] Progress tracking + build logs

### Phase 3.5 — External Deployment (future)
- [ ] Push snapshot to S3 / Cloudflare R2
- [ ] Deploy to Netlify / Vercel via API
- [ ] `rsync` to external VPS
- [ ] GitHub Pages push

> The static snapshot model makes external deployment a simple file-copy step — the platform stays decoupled from delivery infrastructure.

### Phase 4 — Integration
- [ ] Embed published js-tool in course lesson
- [ ] 3D scene folder type → published gallery
- [ ] Shared access links with expiry

---

## Key Files

| File | Role |
|---|---|
| `crates/publications/src/lib.rs` | Publications API + `/pub/{slug}` dispatch |
| `crates/publications/src/serve.rs` | Type-based serving (app/course/presentation) |
| `crates/publications/src/helpers.rs` | Snapshot copy, thumbnail, gallery generation |
| `crates/publications/src/db.rs` | CRUD queries for publications table |
| `crates/standalone/js-tool-viewer/src/lib.rs` | Owner-only workspace file serving (preview) |
| `crates/workspace-manager/src/folder_type_registry.rs` | AppLink struct, folder type definitions |
| `crates/workspace-manager/src/lib.rs` | Workspace browser, "Publish" button |
| `crates/workspace-manager/src/builtin_types/js-tool.yaml` | js-tool folder type |
| `src/apps-catalog.yaml` | Platform-level apps catalog |
| `migrations/20260321120000_publications.sql` | publications table + migration from published_apps |
| `storage-apps/{slug}/` | Published app snapshot (static files) |

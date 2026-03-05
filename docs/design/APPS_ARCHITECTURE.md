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
              storage/apps/{app_id}/     ← static output snapshot
                  ├── index.html
                  └── ...
                         │
                         ▼
            /pub/{app_id}  ──►  end users access
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

Build runs async (like HLS transcoding): spawns a process, tracks progress, output lands in `storage/apps/{app_id}/`.

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

```sql
CREATE TABLE published_apps (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id       TEXT NOT NULL UNIQUE,      -- stable slug, e.g. "my-course"
    workspace_id TEXT NOT NULL,
    folder_path  TEXT NOT NULL,             -- source path within workspace
    folder_type  TEXT NOT NULL,             -- "js-tool", "course", etc.
    user_id      TEXT NOT NULL,
    title        TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    access       TEXT NOT NULL DEFAULT 'private',  -- "public" | "code" | "private"
    access_code  TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Published output stored at: `storage/apps/{app_id}/` (snapshot, separate from workspace).

---

## URL Design

| URL | Purpose |
|---|---|
| `/pub/{app_id}` | Public entry point for a published app |
| `/pub/{app_id}/{*path}` | File serving within the app |
| `/pub/{app_id}?code=XXX` | Access-code protected entry |
| `/api/apps/publish` | POST: publish (copy/build) a workspace folder |
| `/api/apps/{app_id}` | GET / PUT / DELETE: manage a published app |
| `/my-apps` | Owner's dashboard of published apps |

> `/apps` (platform apps page showing 3D gallery, Course Viewer, etc.) is distinct from `/pub/` (user-published apps).

---

## Implementation Progress

### Phase 1 — Foundation ✅ / 🔨
- [x] js-tool viewer crate — serves static files from workspace folders (auth-gated, owner-only)
- [x] AppLink on folder type definitions — connects folder types to app URLs
- [x] "Open App" button in workspace folder detail view
- [x] Folder gallery at `/js-apps/{workspace_id}/{folder}` (owner preview)
- [x] `published_apps` DB table — migration `022_published_apps.sql`
- [x] Publish API: `POST /api/apps/publish` — copy files to `storage/apps/{app_id}/`
- [x] Public serving: `GET /pub/{app_id}` + `GET /pub/{app_id}/{*path}` with access control
- [x] Access control: public + code support on `/pub/` routes (app-publisher crate)
- [x] "Publish / Launch" button in workspace folder detail view (typed folders only)

### Phase 2 — Management
- [x] List API: `GET /api/apps` — user's published apps
- [x] Edit API: `PUT /api/apps/{app_id}` — title, description, access mode
- [x] Delete API: `DELETE /api/apps/{app_id}` — removes snapshot + DB record
- [ ] Re-publish: copy workspace again (overwrite snapshot)
- [x] `/my-apps` dashboard — list published apps with status, URLs, access settings

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
| `crates/standalone/js-tool-viewer/src/lib.rs` | Owner-only workspace file serving (preview) |
| `crates/standalone/app-publisher/src/lib.rs` | Publish API + `/pub/` serving with access control |
| `crates/workspace-manager/src/folder_type_registry.rs` | AppLink struct, folder type definitions |
| `crates/workspace-manager/src/lib.rs` | Workspace browser, "Open App" button |
| `crates/workspace-manager/src/builtin_types/js-tool.yaml` | js-tool folder type |
| `src/apps-catalog.yaml` | Platform-level apps catalog |
| `migrations/022_published_apps.sql` | published_apps table |
| `storage/apps/{app_id}/` | Published app snapshot (static files) |

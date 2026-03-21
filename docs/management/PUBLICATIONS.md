# Publications Registry

> Unified system for publishing and sharing apps, courses, presentations, and media collections under clean `/pub/{slug}` URLs.

---

## Overview

The publications registry replaces the fragmented publishing landscape where apps used `published_apps`, courses used workspace access codes with no registry, and there was no unified way to discover what had been published.

**One table, one URL scheme, one dashboard.**

| Before | After |
|---|---|
| `/pub/app-a1b2c3d4` (apps only) | `/pub/{slug}` (any content type) |
| No course/presentation registry | All types in `publications` table |
| No public catalog | `/catalog` shows all public items |
| `/my-apps` (apps only) | `/my-publications` (all types) |
| Internal IDs in URLs (`workspace_id`, `folder_path`) | Clean slugs derived from title |

---

## Content Types

| Type | Source | What gets served |
|---|---|---|
| **App** | Workspace folder snapshot copied to `storage-apps/{slug}/` | Static files (index.html or gallery) |
| **Course** | Workspace folder (live, not snapshot) | Course viewer with lesson navigation |
| **Presentation** | Workspace folder (live, not snapshot) | Reveal.js presentation viewer |
| **Collection** | Vault media items | Media gallery (future) |

---

## Access Levels

| Level | Who can see it | Catalog | URL |
|---|---|---|---|
| **Public** | Anyone | Listed | `/pub/{slug}` |
| **Code** | Anyone with the code | Not listed | `/pub/{slug}?code=abc123` |
| **Bundled** | Only via a parent publication's code | Not listed | `/pub/{slug}?code={parent's-code}` |
| **Private** | Owner only (session required) | Not listed | `/pub/{slug}` (403 for others) |

The `bundled` level is designed for content that's part of a paid course — it has
no independent access but inherits access from its parent. See
[Publication Bundles](PUBLICATION_BUNDLES.md) for details.

---

## Slug System

Slugs are derived from the publication title:

- `"Intro to Rust"` → `intro-to-rust`
- `"Hello, World!"` → `hello-world`
- On conflict: appends `-2`, `-3`, etc.
- User can provide a custom slug at creation time
- Migrated apps keep their `app-{hex}` ID as slug

---

## Pages

### `/catalog` — Public Catalog

Grid of cards showing all public publications. Filter tabs: All / Apps / Courses / Presentations / Collections. No auth required.

### `/my-publications` — Admin Dashboard

Lists all of the user's publications across all access levels. Actions per item:

- **Open** — view the publication
- **Copy** — copy shareable link (includes `?code=` for code-gated items)
- **Edit** — change title, description, access level, regenerate code, upload thumbnail
- **Republish** — refresh app snapshot from workspace (app type only)
- **Delete** — unpublish and remove snapshot

---

## Publishing Flows

### App Publishing

1. User calls `POST /api/publications` with `pub_type: "app"`, workspace_id, folder_path, title
2. System validates workspace ownership
3. Workspace folder is copied to `storage-apps/{slug}/`
4. Thumbnail auto-detected and converted
5. Gallery marker created if no `index.html` exists
6. Publication record inserted
7. Returns `{ slug, url: "/pub/{slug}", access_code }`

### Course / Presentation Publishing

1. User calls `POST /api/publications` with `pub_type: "course"`, workspace_id, folder_path, title
2. System validates workspace ownership
3. A workspace access code is always created (required for file serving)
4. If access is `"code"`, the access code is returned to the user
5. Publication record inserted with access_code and source pointers
6. `/pub/{slug}` serves the course viewer directly — no workspace_id in the URL

### Republishing (App only)

`POST /api/publications/{slug}/republish` — removes old snapshot, re-copies from workspace folder, regenerates thumbnail.

---

## Backward Compatibility

- Existing apps migrated from `published_apps` keep their `app-{hex}` slug via `legacy_app_id`
- The `published_apps` table is not dropped — the migration copies data into `publications`
- The old `/pub/{app-hex}` URLs continue to work because migrated slugs match the legacy ID
- `/my-apps` links have been updated to `/my-publications`

---

## Relation to Existing Systems

| System | Role | Interaction |
|---|---|---|
| `published_apps` table | Legacy app registry | Migrated into `publications` at migration time |
| `workspace_access_codes` | File serving gate for courses | Publications creates codes when publishing courses/presentations |
| `publications` crate `helpers` module | App snapshot creation + serving helpers | Contains `copy_dir_recursive`, `generate_gallery_index`, thumbnail conversion (consolidated from former `app-publisher` crate) |
| `course` crate | Course/presentation rendering | Publications calls `render_public_course()` / `render_public_presentation()` |

---

## Data Model

```sql
CREATE TABLE publications (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    slug            TEXT NOT NULL UNIQUE,
    user_id         TEXT NOT NULL,
    pub_type        TEXT NOT NULL,       -- 'app' | 'course' | 'presentation' | 'collection'
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    access          TEXT NOT NULL DEFAULT 'private',
    access_code     TEXT,
    workspace_id    TEXT,
    folder_path     TEXT,
    vault_id        TEXT,
    legacy_app_id   TEXT,
    thumbnail_url   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## Related Docs

- `docs/management/PUBLICATION_BUNDLES.md` — access inheritance for bundled content (courses embedding apps)
- `docs/apps/course-viewer.md` — course viewer and publishing modes
- `docs/apps/course-app-embed.md` — embed syntax for apps, images, videos, presentations

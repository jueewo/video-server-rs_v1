# Publications Architecture

> Technical design of the unified publications registry.

---

## Crate: `crates/publications/`

```
crates/publications/
  Cargo.toml
  askama.toml          # Template dirs: own + project root
  src/
    lib.rs             # PublicationsState, Router, API handlers
    db.rs              # CRUD queries against publications table
    serve.rs           # /pub/{slug} dispatch + access gating
    slug.rs            # Slug generation + uniqueness
    helpers.rs         # Shared utilities: thumbnail, copy, gallery, MIME, ID generation
  templates/
    publications/
      catalog.html          # Public catalog grid
      my_publications.html  # Admin dashboard (extends base-tailwind.html)
      access_denied.html    # Code-gated entry form
```

---

## State

```rust
pub struct PublicationsState {
    pub pool: SqlitePool,
    pub storage_base: PathBuf,       // workspace roots
    pub apps_dir: PathBuf,           // storage-apps/ for app snapshots
    pub user_storage: UserStorageManager,
}
```

Created in `workspace_app_routes()` and passed alongside `JsToolViewerState`.

---

## Route Table

### Admin API (auth required)

| Method | Path | Handler | Purpose |
|---|---|---|---|
| `POST` | `/api/publications` | `create_handler` | Create publication |
| `GET` | `/api/publications` | `list_handler` | List user's publications |
| `GET` | `/api/publications/find` | `find_handler` | Find by workspace_id+folder_path |
| `PUT` | `/api/publications/{slug}` | `update_handler` | Update title/description/access |
| `DELETE` | `/api/publications/{slug}` | `delete_handler` | Unpublish + delete |
| `POST` | `/api/publications/{slug}/republish` | `republish_handler` | Refresh app snapshot |
| `POST` | `/api/publications/{slug}/thumbnail` | `upload_thumbnail_handler` | Upload thumbnail |
| `GET` | `/api/publications/{slug}/thumbnail` | `serve_api_thumbnail_handler` | Serve thumbnail (auth) |

### Public

| Method | Path | Handler | Purpose |
|---|---|---|---|
| `GET` | `/pub/{slug}` | `serve_publication` | Serve by type dispatch |
| `GET` | `/pub/{slug}/` | `serve_publication` | Same (trailing slash) |
| `GET` | `/pub/{slug}/thumbnail` | `serve_publication_thumbnail` | Serve thumbnail (no auth) |
| `GET` | `/pub/{slug}/{*path}` | `serve_publication_file` | Serve sub-files (app snapshots) |

### Pages

| Method | Path | Handler | Purpose |
|---|---|---|---|
| `GET` | `/catalog` | `catalog_handler` | Public catalog |
| `GET` | `/my-publications` | `my_publications_handler` | Admin dashboard |

---

## `/pub/{slug}` Dispatch Logic

```
1. Look up publication by slug
2. Access gate:
   - public → pass
   - code → check ?code= query param
   - private → return 403 (session auth not yet integrated)
3. Dispatch by pub_type:
   ┌──────────────┬──────────────────────────────────────────────┐
   │ app           │ Serve static files from apps_dir/{slug}/    │
   │               │ Gallery index if no index.html              │
   │ course        │ course::render_public_course()              │
   │ presentation  │ course::render_public_presentation()        │
   │ collection    │ (not yet implemented)                       │
   └──────────────┴──────────────────────────────────────────────┘
```

---

## Internal Modules

### `helpers.rs` — Shared Utilities

Consolidated from the former `app-publisher` crate. These functions live directly in the publications crate:

| Function | Purpose |
|---|---|
| `copy_dir_recursive()` | Copy workspace folder to snapshot dir |
| `find_thumbnail_in_dir()` | Detect thumbnail image in source folder |
| `convert_image_to_thumb()` | Resize image to 512x512 JPEG thumbnail |
| `convert_bytes_to_thumb()` | Convert uploaded bytes to thumbnail |
| `generate_gallery_index()` | Generate HTML gallery for snapshot with subdirs |
| `mime_for_path()` | MIME type detection by file extension |
| `generate_app_id()` | Generate `app-{hex}` IDs (legacy compat) |
| `generate_access_code()` | Generate hex access codes |

## Cross-Crate Dependencies

### From `course` (render helpers)

| Function | Purpose |
|---|---|
| `render_public_course()` | Render course viewer template for a resolved workspace folder |
| `render_public_presentation()` | Render presentation viewer template |

These extract the core rendering logic from the existing `course_viewer_handler` and `presentation_viewer_handler` — loading course structure, resolving branding, rendering template — into reusable functions that the publications dispatcher calls.

---

## Database Schema

Migration: `migrations/20260321120000_publications.sql`

**Indexes:**
- `idx_publications_user_id` — list user's publications
- `idx_publications_pub_type` — filter by type
- `idx_publications_access` — catalog query (WHERE access = 'public')

**Data migration:** existing `published_apps` rows are inserted into `publications` with `slug = app_id` and `legacy_app_id = app_id`.

---

## Slug Generation

`slug.rs` provides two functions:

- `slugify(title)` — lowercases, replaces non-alphanumeric with `-`, collapses dashes
- `ensure_unique_slug(pool, base_slug)` — queries DB, appends `-2`, `-3` on conflict

---

## Workspace Access Codes for Courses

When publishing a course or presentation, the handler always creates a `workspace_access_code` + `workspace_access_code_folders` entry. This is required because the course viewer's file serving (images, workspace files) validates requests against workspace access codes.

The publication's `access` field controls *discovery* (catalog visibility, access gate). The workspace access code controls *file serving*.

---

## Wiring into Main App

```
src/main.rs
  → workspace_app_routes(pool, storage_dir, apps_dir, user_storage)
    → PublicationsState created
    → publications_routes(pub_state)
    → js_tool_viewer_routes(js_state)
    → gallery3d::router(pool)
```

All `/pub/{slug}` serving and `/api/publications/*` endpoints are handled by the publications crate. The former `app-publisher` crate has been fully consolidated into publications. Legacy apps still work because their migrated slugs match the legacy `app-{hex}` format.

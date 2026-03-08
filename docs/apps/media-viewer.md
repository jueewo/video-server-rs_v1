# Media Viewer (Gallery)

> `crates/media-viewer/` — dual-use crate.
> Workspace-embedded media grid + standalone public gallery accessible via access code.

**Status:** Implemented 2026-03-08

---

## Overview

The media viewer surfaces vault-backed media items in two contexts from the same crate.
In the workspace browser it renders as an authenticated grid inside a `media-server` folder.
Via an access code it renders as a public gallery — no login required.

Two modes from the same crate:

| Mode | URL | Auth |
|---|---|---|
| Embedded | Open a `media-server` folder in workspace browser | Session (owner) |
| Standalone | `GET /gallery?code={code}` | Access code only |

---

## Embedded Mode (Workspace Browser)

1. In the workspace browser, open any folder.
2. Click the Settings gear on the folder card.
3. Set folder type to **Media Server**.
4. Open the folder — the media grid renders inline:
   - Responsive grid (2–5 columns) of thumbnail cards
   - Each card links to the media detail page (`/media/{slug}`)
   - **Upload** button pre-fills `vault_id` on the upload form
   - **Share** button opens the access-code modal (creates a `vault_id` grant)

The embedded view queries `media_items` filtered to the folder's vault and the
current user — no cross-vault leakage.

---

## Standalone Mode (Public Gallery)

### 1. Create an access code

In the workspace browser, click the **Share** button on a `media-server` folder.

- Enter a memorable code (e.g. `marketing-q1-2026`)
- Optionally add a description and expiry
- Copy the **Gallery URL** shown in the success panel

Or via API:
```bash
curl -X POST http://localhost:3000/api/access-codes \
  -H 'Content-Type: application/json' \
  -d '{
    "code": "marketing-q1-2026",
    "description": "Q1 campaign assets — external review",
    "vault_id": "vault-a1b2c3d4"
  }'
```

### 2. Share the URL

```
https://your-platform.example.com/gallery?code=marketing-q1-2026
```

The recipient opens this in any browser — no account, no login.

### 3. What the recipient sees

- Full-page gallery with type filter buttons (All / Images / Videos / Documents)
- Responsive grid (2–6 columns) of thumbnail cards showing title, type badge, file size
- **Image** cards → lightbox overlay with full image
- **Video** cards → in-page modal player
- **Document** cards → PDF opens in a new tab
- Invalid or expired code → 404

All media is served via the standard `/media/{slug}/...` endpoints with `?code=` appended
— the same authorization path used everywhere else.

### 4. Filter by type

The type filter buttons (All / Images / Videos / Documents) work client-side with no
page reload — useful when a vault contains a mix of media types.

---

## Revoking Access

Go to **Workspace Access Codes** and revoke the code.
The gallery URL stops working immediately. The vault and its media are unaffected.
Create a new code at any time to re-share.

---

## Developer Reference

### Crate layout

```
crates/media-viewer/
  src/
    lib.rs           ← MediaViewerState, GalleryItem, gallery_routes(),
                       MediaViewerRenderer (FolderTypeRenderer)
  templates/
    media-viewer/
      viewer.html    ← standalone full-page gallery (filter buttons, lightbox, video modal)
      folder.html    ← embedded workspace grid
  askama.toml        ← dirs = ["templates", "../../templates"]
  Cargo.toml
```

No dependency on `media-manager` — queries `media_items` directly with its own pool.

### Key types

```rust
pub struct MediaViewerState {
    pub pool: SqlitePool,
    pub storage: UserStorageManager,
}

pub struct GalleryItem {
    pub slug: String,
    pub title: String,
    pub media_type: String,          // "video" | "image" | "document"
    pub thumbnail_url: Option<String>,
    pub file_size_str: String,
}
```

`GalleryItem` helper methods:
- `type_label()` → `"Video"` / `"Image"` / `"Document"`
- `type_icon()` → Lucide icon name (`"clapperboard"` / `"image"` / `"file-text"`)
- `serve_url(code)` → type-appropriate serving URL with `?code=` appended
- `thumb_url(code)` → thumbnail URL with `?code=` appended

### Wiring in main.rs

```rust
use media_viewer::{gallery_routes, MediaViewerRenderer, MediaViewerState};

// Register embedded renderer (replaces the old MediaFolderRenderer from media-manager)
workspace_state.register_renderer(Arc::new(MediaViewerRenderer { pool: pool.clone() }));

// Register standalone gallery routes
let mv_state = Arc::new(MediaViewerState {
    pool: pool.clone(),
    storage: (*user_storage).clone(),
});
// ... in router:
.merge(gallery_routes(mv_state))
```

### Standalone route

```
GET /gallery?code={code}    →  public gallery for all vaults granted by this code
```

The handler:
1. Resolves `code` → `Vec<vault_id>` via `workspace_access_code_folders` (vault grants only)
2. For each vault, queries `media_items WHERE vault_id = ? AND status = 'active'`
3. Collects into `Vec<GalleryItem>`, returns 404 if empty
4. Renders `viewer.html`

### Embedded renderer

`MediaViewerRenderer::render_folder_view()`:
1. Reads `vault_id` from `ctx.meta_str("vault_id")` — set automatically when the folder type is assigned
2. Queries `media_items WHERE vault_id = ? AND user_id = ? AND status = 'active'`
3. Renders `folder.html`

The `type_id()` is `"media-server"` — matches the folder type registry entry.

### SQL — access code → vault IDs

```sql
SELECT f.vault_id
FROM workspace_access_codes wac
JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
WHERE wac.code = ? AND wac.is_active = 1
  AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
  AND f.vault_id IS NOT NULL
```

---

## Migration from media-manager

Prior to 2026-03-08, the embedded media grid was implemented in
`crates/media-manager/src/folder_renderer.rs` as `MediaFolderRenderer`.
That module has been deleted. `MediaViewerRenderer` is a drop-in replacement:
same `type_id()`, same DB query, same template structure — moved to the clean
dual-use crate that also handles the standalone gallery.

---

## See Also

- `docs/apps/DUAL_USE_PATTERN.md` — how dual-use crates work in general
- `docs/management/media-server-folder-type.md` — workspace folder type setup
- `docs/management/WORKSPACE_ACCESS_CODES.md` — access code API
- `crates/workspace-core/src/lib.rs` — `FolderTypeRenderer` trait

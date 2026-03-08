# Media-Server as a Workspace Folder Type

**Status:** Implemented (2026-03-06) | Gallery sharing added (2026-03-08)

## Overview

The `media-server` folder type unifies the workspace file browser with the vault-backed
media pipeline at the UX level, without a major architectural rewrite.

**Guiding principle:** The folder type is the pipeline declaration. Drop a file into a
`media-server` folder → it gets processed (WebP conversion, HLS transcoding, thumbnail
generation, access codes, serving endpoints). Drop it anywhere else in the workspace →
it's just a file.

## Design

### Clean Split

| Location | Treatment | File types |
|---|---|---|
| Workspace filesystem | Plain files — git-trackable, WebDAV, human-readable | `.md`, `.bpmn`, `.yaml`, `.svg`, `.html`, `.js`, PDFs for internal use |
| Vault (via media-server folder) | Pipeline-processed — thumbnails, transcoding, sharing | Images, videos, PDFs to share |

### Vault as Implementation Detail

**Vault stays as the implementation detail.** A workspace folder with
`folder-type: media-server` is a named view into a vault. Files physically live in
`storage/vaults/{vault_id}/`. Nothing moves. One DB. Access codes work unchanged.

---

## What Was Implemented

### 1. Folder Type Registry Entry
**File:** `storage/folder-type-registry/media-server.yaml`

Registers the `media-server` type with the icon, color, and description shown in the
folder type picker.

### 2. Auto-Create Vault on Type Assignment
**File:** `crates/workspace-manager/src/lib.rs` — `update_folder_metadata` handler

When a folder's type is set to `"media-server"`:
1. Checks `metadata.vault_id` — skips if already set
2. Creates a new vault via DB insert + `storage.ensure_vault_storage(vault_id)`
3. Writes `vault_id` into `FolderConfig.metadata` and saves `workspace.yaml`

Result in `workspace.yaml`:
```yaml
folders:
  marketing-assets:
    type: media-server
    metadata:
      vault_id: "vault-a1b2c3d4"
```

### 3. Embedded Media Grid
**Crate:** `crates/media-viewer/` — `MediaViewerRenderer`

When a `media-server` folder is opened in the workspace browser:
- Queries `media_items` filtered to the folder's `vault_id` and current user
- Renders a responsive thumbnail grid (2–5 columns)
- Each card links to the media detail page (`/media/{slug}`)
- **Upload** button pre-fills `vault_id` on the upload form
- **Share** button opens the access-code modal

> Prior to 2026-03-08, this was `MediaFolderRenderer` inside `media-manager`.
> It has been moved to the dedicated `media-viewer` crate.

### 4. Upload Within Folder → Vault Pre-Selected
When the media-server UI loads with a `vault_id` context, the upload form pre-populates
`vault_id` (hidden field). Already supported by the existing upload handler.

---

## Sharing — Public Gallery

The **Share** button on a `media-server` folder creates a workspace access code that
grants access to the folder's vault. The success panel shows two URLs:

| URL | Use |
|---|---|
| `GET /gallery?code={code}` | Public gallery — images, videos, documents in a browser |
| `GET /api/folder/{code}/media` | JSON list — for satellite apps, 3D gallery, scripts |

The gallery is served by `crates/media-viewer/` with no session required.
All media items (`/media/{slug}/image.webp`, `/media/{slug}/video.mp4`, etc.) are
served with the code appended — existing serve-route authorization handles it.

To revoke: go to **Workspace Access Codes** and revoke the code.
The vault and media are unaffected; a new code can be created at any time.

---

## What Did NOT Change

- `storage_vaults` DB table — unchanged
- `media_items.vault_id` — unchanged
- `storage/vaults/{vault_id}/...` paths — unchanged
- Access codes — scoped to `created_by` user + vault, work as-is
- HLS transcoding pipeline — vault_id already flows through
- All media-manager routes and handlers

---

## Verification Checklist

1. Create a workspace, assign a folder to `folder-type: media-server`
2. Confirm `workspace.yaml` gains `vault_id` under that folder's metadata
3. Confirm a new row appears in `storage_vaults`
4. Open the folder — thumbnail grid renders inline (no redirect)
5. Upload an image/video — confirm it lands under `storage/vaults/{vault_id}/`
6. Create a second media-server folder — confirm a separate vault is created, media is isolated
7. Click **Share**, create a code, copy the gallery URL
8. Open the gallery URL in a private/incognito window — gallery renders without a session
9. Click an image → lightbox; click a video → modal player; click a document → new tab
10. Revoke the code — gallery returns 404

---

## Architecture Notes

The embedded renderer (`MediaViewerRenderer`) and the standalone gallery handler
live together in `crates/media-viewer/` following the dual-use crate pattern.
See `docs/apps/media-viewer.md` for the full developer reference.

---

## Future Considerations

- **Phase 1 (ROADMAP):** Long-term goal is to retire vault as a user-facing concept and
  merge workspace + vault storage. The `media-server` folder type is a pragmatic UX bridge
  that makes the system coherent today while that migration is planned.
- Uploading directly from a media-server folder (drag-drop into browser → goes to vault)
  is a future UX enhancement.

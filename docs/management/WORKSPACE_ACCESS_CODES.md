# Workspace Access Codes — User Guide

> How to share workspace folders with external apps, clients, and other users.

---

## Overview

A workspace access code is a short string that grants access to one or more workspace folders — without requiring the recipient to have an account on the platform.

**Two use cases:**

1. **External / satellite app access** — pass `?code=` in API calls. No session required.
2. **Internal user sharing** — another logged-in user claims the code and sees the content alongside their own.

---

## Creating a Code

1. Open a workspace and browse to the folder you want to share.
2. Click the **Share** button (share icon) on the folder card.
3. The modal has two tabs:

**New code tab:**
- **Code** — auto-generated, editable. Choose something memorable for satellite apps (e.g. `course1-code`).
- **Description** — optional note for yourself (editable later on the management page).
- **Expiry** — optional. Leave blank for permanent access.
- Submit → the code is created and copied to clipboard.

**Add to existing tab:**
- Dropdown lists all your active codes with their descriptions and folder counts.
- Select one, then click "Add folder to code" — the current folder is linked to that code immediately.

The code is active immediately.

### Media-server folders and group scoping

When sharing a `media-server` folder, you can optionally set a **group** in the share modal. If set, only media items belonging to that group are exposed — useful when the same vault serves multiple apps that should each see a different subset.

---

## Editing a Code

On the **/workspace-access-codes** management page, hover the **Description** column of any code you created — a pencil icon appears. Click it to edit inline. Press Enter to save or Escape to cancel.

The **Folders** column shows every folder linked to the code as path badges — e.g. `workspace-886fd040/processes`. To add more folders, use the **Add to existing** tab in the Share modal on any folder card.

---

## Using a Code — External / Satellite App

### Get media items

```
GET /api/folder/{code}/media
```

Returns all active media items from every `media-server` folder linked to the code:

```json
{
  "code": "my-code",
  "items": [
    {
      "slug": "intro-video",
      "title": "Introduction",
      "media_type": "video",
      "mime_type": "video/mp4",
      "file_size": 10487647,
      "serve_url": "/media/intro-video/video.mp4?code=my-code",
      "thumbnail_url": "/media/intro-video/thumbnail"
    }
  ]
}
```

No session cookie required. The code is the credential.

### Get files (BPMN, docs, course, etc.)

```
GET /api/folder/{code}/files
```

Returns all files **recursively** from every plain folder linked to the code. The `name` field is the relative path from the folder root (e.g. `session1/chapter1/getting_started.md`):

```json
{
  "code": "my-code",
  "folders": [
    {
      "workspace_id": "ws-abc",
      "folder_path": "course1",
      "files": [
        {
          "name": "session1/chapter1/getting_started.md",
          "size": 348,
          "serve_url": "/api/workspaces/ws-abc/files/serve?path=course1%2Fsession1%2Fchapter1%2Fgetting_started.md&code=my-code"
        }
      ]
    }
  ]
}
```

### Serve individual files/media (no session)

All serving endpoints accept `?code=` in place of a session:

```
GET /media/{slug}/image.webp?code=my-code
GET /media/{slug}/thumbnail?code=my-code
GET /media/{slug}/video.mp4?code=my-code
GET /media/{slug}/serve?code=my-code          (PDF)
GET /api/workspaces/{id}/files/serve?path=...&code=my-code
```

---

## Using a Code — Internal User (Claiming)

If you receive a code and have an account:

1. Go to **/workspace-access-codes**.
2. Enter the code under "Claim a code".
3. The shared folders appear in your "Shared with me" section.
4. To stop seeing them, unclaim the code from the same page.

---

## Revoking a Code

On the **/workspace-access-codes** management page, click **Deactivate** next to any code you created.

**Important:** Deactivating a code only removes **that access path**. If the same folder is covered by another active code (e.g., you shared it with two different satellite apps), those apps retain access. Each code has an independent lifecycle.

---

## Managing Codes

**`/workspace-access-codes`** — your management hub:

- **Codes you created** — code, description (editable inline), folder path badges, expiry, status, Revoke button.
- **Codes you've claimed** — code, description, who created it, when claimed, Unclaim button.
- **Claim a code** — input at the top to claim a code shared with you.

### API endpoints for management (session required)

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/workspace-access-codes` | Create a code |
| `GET` | `/api/workspace-access-codes` | List codes you created |
| `PATCH` | `/api/workspace-access-codes/{code}` | Update description and/or expires_at |
| `DELETE` | `/api/workspace-access-codes/{code}` | Deactivate a code |
| `POST` | `/api/workspace-access-codes/{code}/folders` | Add a folder to an existing code |
| `POST` | `/api/workspace-access-codes/claim` | Claim a code |
| `DELETE` | `/api/workspace-access-codes/{code}/claim` | Unclaim a code |

---

## FAQ

**Q: Can a media item be in multiple codes?**
Yes. The same item (or the same folder) can be covered by any number of active codes. Revoking one does not affect the others.

**Q: Does a code expose vault IDs?**
No. The API responses use workspace/folder language. Vault IDs are resolved internally and never returned.

**Q: What if I share a folder that later gets new files?**
New files added to the folder are immediately accessible to anyone holding the code — no code update needed. This is by design.

**Q: Can I share a subset of a media-server folder?**
Yes — use the group_id option when creating the code. Only items in that group are exposed.

**Q: Can a satellite app use the code for both media and files?**
Yes, if the code covers both a media-server folder and a plain folder. Call `/api/folder/{code}/media` for media items and `/api/folder/{code}/files` for raw files.

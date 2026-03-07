# Access Codes

> Audience: developers, satellite app builders, power users.

Two separate systems exist. They are not interchangeable.

---

## Two Systems

```
┌─────────────────────────────────────────────────────────────────┐
│  WORKSPACE ACCESS CODES  (workspace_access_codes)  ← USE THIS  │
│                                                                 │
│  Share a workspace FOLDER with:                                 │
│  • External apps / clients — no account needed (?code= URL)    │
│  • Internal users — claim the code, browse shared content      │
│                                                                 │
│  One code can cover multiple folders:                           │
│  • media-server folder → all media items (optional group scope) │
│  • BPMN / docs / course folder → raw files                     │
│                                                                 │
│  UI: Share button on any folder in the workspace browser        │
│  Management: /workspace-access-codes                            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  PER-ITEM ACCESS CODES  (access_codes)  ← gallery3d only       │
│                                                                 │
│  Grant access to a hand-curated list of specific media slugs.  │
│  Used when you want to expose exactly N items — not a folder.  │
│                                                                 │
│  Currently: managed via API only (no UI).                       │
│  One media item can appear in many per-item codes at once.     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Workspace Access Codes in Detail

### What they cover

A workspace access code is linked to one or more **folder grants**. Each grant is a `(workspace_id, folder_path)` pair:

- **`media-server` folder** → exposes all active media items in the underlying vault. Optional `group_id` scopes the grant to items in a specific group only.
- **Any other folder** (BPMN, docs, course, etc.) → exposes the raw files in that filesystem directory.

Multiple folders can be covered by one code. Example: a satellite app gets one code that covers a BPMN folder (for diagrams) and a media-server folder (for associated videos).

### Many codes per item

A media item can be covered by multiple active codes simultaneously. Revoking (deactivating) one code does **not** remove access granted by another active code covering the same folder. This is intentional: the same media-server folder can be shared with different apps or users under different codes, each with independent lifecycle.

### No vault language

Workspace access codes reference workspace folders. The vault behind a `media-server` folder is resolved internally and never exposed to code holders or satellite apps. Satellite apps call `GET /api/folder/{code}/media` and get back media items with serving URLs — no vault ID in the response.

### Adding folders to an existing code

A code can cover multiple folders. To add a folder to an existing code:

- **UI:** Share button on the folder → "Add to existing" tab → select code from dropdown → "Add folder to code"
- **API:** `POST /api/workspace-access-codes/{code}/folders` with `{ workspace_id, folder_path }`

The folder is added immediately. `INSERT OR IGNORE` means adding a folder that is already linked is a no-op.

### Editing a code

- **Description and expiry** can be updated via `PATCH /api/workspace-access-codes/{code}` or inline on the management page (hover the Description column → pencil icon).
- **Deactivating** sets `is_active = 0`. The code stops working immediately for all holders. This cannot be reversed via the UI (re-activation requires a direct DB update).

### Internal user claiming

An authenticated user can claim a workspace access code. This adds a `user_claimed_workspace_codes` row. Claimed codes give the user read access to the shared folders alongside their own content ("Shared with me"). Unclaiming removes the row.

---

## Per-Item Access Codes in Detail

Per-item codes (`access_codes` + `access_code_permissions`) grant access to a specific set of media slugs selected individually. This is useful when you want to expose a curated subset from a vault — for example, 3 sample videos from a larger collection — without exposing the whole folder.

The 3D gallery uses this model: the gallery configuration names specific slugs; a per-item code grants the gallery app access to those exact items.

**When to use per-item vs workspace folder code:**

| Scenario | Use |
|---|---|
| Share all items in a media-server folder | Workspace access code |
| Share items from a folder, scoped to one group | Workspace access code + group_id |
| Share exactly 3 specific items from anywhere | Per-item code |
| 3D gallery | Per-item code (required by gallery config) |

---

## Open Question: Per-Item Code UI

Per-item codes currently have no creation UI. They are provisioned via API or direct DB insert.

Given the ROADMAP direction of hiding the standalone `/media` page and routing all media access through the workspace browser, the natural future home for per-item code management is the **media-server folder inline view** — a "Share item" button on individual media items within the folder grid. Not a separate page, not accessible from `/media` directly.

This is not in Phase 1. Captured here for planning.

---

## See Also

- `MENTAL_MODEL.md` — workspace/vault/pipeline architecture
- `WORKSPACE_ACCESS_CODES.md` — user guide for creating and using workspace access codes
- `ROADMAP.md` — Phase 1 (access model), Phase 3 (open access layer)

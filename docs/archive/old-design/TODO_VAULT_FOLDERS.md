# TODO: Vault Folders

**Status:** Idea / Not started
**Created:** 2026
**Context:** Design discussion — no code written yet

> **Note:** This is about hierarchical folders WITHIN vaults for organizing published content.
> **Workspaces** (personal file management) are already implemented — see `WORKSPACES_STATUS.md`.

---

## Problem

Vaults organise files by owner/privacy boundary. Groups handle sharing.
For larger collections (100s of files) there is no way to sub-organise items
*within* a vault — everything is a flat list. A folder hierarchy solves this.

**Example:** A vault contains 200 BPMN diagrams — currently they're a flat list.
With vault folders, they could be organized by department/project/year.

---

## Settled Design Decisions

| Decision | Choice |
|---|---|
| Groups | Stay as pure sharing/access mechanism — unchanged |
| Folders | New `vault_folders` table — hierarchical, vault-scoped |
| Physical files on disk | Stay flat (keyed by slug + vault_id) — no disk restructuring ever |
| Folder ↔ sharing | Each folder can optionally have an `access_group_id` and `access_code_id` |
| `folder_id = NULL` | Item lives at vault root — existing items need zero migration |
| UI touch points | Upload form (assign folder on upload) + Media filter (browse by folder) |
| Nesting | Unlimited in DB — UI can cap at 3–4 levels |
| Soft-delete | `is_archived` flag; real delete cascades via FK to children and media items |

---

## Example Use Case: BPMN Repository

A team stores hundreds of BPMN process diagrams in one vault. Without folders
it becomes an unmanageable flat list. With folders:

```
vault: "Engineering Processes"
 ├── Onboarding/
 │    ├── employee-onboarding.bpmn
 │    └── contractor-onboarding.bpmn
 ├── Finance/
 │    ├── invoice-approval.bpmn
 │    └── expense-claim.bpmn
 └── Releases/
      ├── 2024/
      │    └── release-checklist.bpmn
      └── 2025/
           └── release-checklist.bpmn
```

**Sharing:** The `Finance/` folder has `access_group_id` pointing at the
"Finance Team" group — members of that group can browse and view all BPMNs
under that subtree without touching anything else in the vault.

**API access:** The `Releases/` folder has an `access_code_id` — an external
CI/CD pipeline authenticates with that code to fetch release BPMNs
programmatically, scoped to that folder only.

**Both at once:** A folder can carry both a group and an access code — humans
browse via group membership, automated systems use the access code.

---

## DB Schema

### `vault_folders` — new table

```sql
CREATE TABLE vault_folders (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Which vault this folder belongs to
    vault_id        TEXT    NOT NULL
                    REFERENCES storage_vaults(vault_id) ON DELETE CASCADE,

    -- NULL = direct child of vault root
    parent_id       INTEGER
                    REFERENCES vault_folders(id) ON DELETE CASCADE,

    -- Display name, e.g. "Project X / 2024"
    name            TEXT    NOT NULL,

    -- Materialized path of the *parent*, always without leading/trailing slash.
    -- Root children: path = ""
    -- e.g. for a folder at root:           path = "",        full = "photos"
    -- e.g. for a nested folder:            path = "photos",  full = "photos/2024"
    path            TEXT    NOT NULL DEFAULT '',

    description     TEXT,

    -- Optional sharing: attach an access group to this folder
    -- Members of this group can see/access all items in the folder subtree
    access_group_id INTEGER
                    REFERENCES access_groups(id) ON DELETE SET NULL,

    -- Optional: a specific access code that unlocks this folder
    access_code_id  INTEGER
                    REFERENCES access_codes(id) ON DELETE SET NULL,

    is_archived     INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    -- Folder name must be unique within its parent in the same vault
    UNIQUE (vault_id, COALESCE(parent_id, -1), name)
);
```

**Indexes:**

```sql
CREATE INDEX idx_vault_folders_vault_id    ON vault_folders(vault_id);
CREATE INDEX idx_vault_folders_parent_id   ON vault_folders(parent_id);
CREATE INDEX idx_vault_folders_path        ON vault_folders(vault_id, path);
CREATE INDEX idx_vault_folders_group       ON vault_folders(access_group_id);
CREATE INDEX idx_vault_folders_active      ON vault_folders(vault_id, is_archived)
    WHERE is_archived = 0;
```

### `media_items` — one new column

```sql
-- NULL = item lives at vault root (no migration needed for existing items)
ALTER TABLE media_items ADD COLUMN folder_id INTEGER
    REFERENCES vault_folders(id) ON DELETE SET NULL;

CREATE INDEX idx_media_items_folder_id   ON media_items(folder_id);
CREATE INDEX idx_media_items_folder_type ON media_items(folder_id, media_type);
```

---

## Rust Model

```rust
// crates/vault-manager/src/folders.rs  (new file)

pub struct VaultFolder {
    pub id:              i32,
    pub vault_id:        String,
    pub parent_id:       Option<i32>,     // None = vault root
    pub name:            String,
    pub path:            String,          // materialized parent path, e.g. "photos/2024"
    pub description:     Option<String>,
    pub access_group_id: Option<i32>,     // optional group that can access this folder
    pub access_code_id:  Option<i32>,     // optional access code for this folder
    pub is_archived:     bool,
    pub created_at:      String,
    pub updated_at:      String,
}

/// Full path of this folder (path + "/" + name, or just name at root)
impl VaultFolder {
    pub fn full_path(&self) -> String {
        if self.path.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", self.path, self.name)
        }
    }
}
```

---

## Key Operations

```rust
// List root folders of a vault
fn list_root(vault_id) -> Vec<VaultFolder>

// List direct children of a folder
fn list_children(folder_id) -> Vec<VaultFolder>

// List media items directly in a folder (folder_id = None → vault root)
fn list_items(vault_id, folder_id: Option<i32>) -> Vec<MediaItem>

// Create a folder (path derived from parent at insert time)
fn create(vault_id, parent_id: Option<i32>, name, description, group_id, code_id) -> VaultFolder

// Rename a folder (update path for all descendants)
fn rename(folder_id, new_name)

// Move a folder to a new parent (update parent_id + bulk-fix descendant paths)
fn move_folder(folder_id, new_parent_id: Option<i32>)

// Move a media item into a folder (or back to root)
fn move_item(media_id, folder_id: Option<i32>)

// Recursive subtree — used for delete / bulk operations
fn subtree(root_folder_id) -> Vec<VaultFolder>  // recursive CTE
```

---

## Useful Query Patterns

**List root folders:**
```sql
SELECT * FROM vault_folders
WHERE vault_id = ? AND parent_id IS NULL AND is_archived = 0
ORDER BY name;
```

**List direct children:**
```sql
SELECT * FROM vault_folders
WHERE vault_id = ? AND parent_id = ? AND is_archived = 0
ORDER BY name;
```

**Everything under a subtree (materialized path — fast):**
```sql
SELECT * FROM vault_folders
WHERE vault_id = ?
  AND (path = ? OR path LIKE ? || '/%')
  AND is_archived = 0;
-- bind: vault_id, full_path_of_parent, full_path_of_parent
```

**Recursive subtree via CTE (for delete / move):**
```sql
WITH RECURSIVE subtree AS (
    SELECT * FROM vault_folders WHERE id = :root_id
    UNION ALL
    SELECT f.* FROM vault_folders f
    JOIN subtree s ON f.parent_id = s.id
)
SELECT * FROM subtree;
```

**Rename / move — bulk-fix descendant paths:**
```sql
UPDATE vault_folders
SET path       = replace(path, :old_prefix, :new_prefix),
    updated_at = datetime('now')
WHERE vault_id = ? AND path LIKE :old_prefix || '%';
```

**Item counts per folder (UI badges):**
```sql
SELECT folder_id, COUNT(*) AS item_count
FROM media_items
WHERE vault_id = ?
GROUP BY folder_id;
```

---

## Routes Sketch

```
GET    /vaults/:vault_id/folders                    list root folders + item count badges
GET    /vaults/:vault_id/folders/:folder_id         list children + items inside folder
POST   /vaults/:vault_id/folders                    create folder
PATCH  /vaults/:vault_id/folders/:folder_id         rename / move / update group+code
DELETE /vaults/:vault_id/folders/:folder_id         delete (cascades to children + unlinks items)
PATCH  /media/:slug/folder                          move item to folder (or root)
```

---

## UI Touch Points

Only two places need folder awareness to cover the main workflow:

### 1. Upload form (`/media/upload`)

- Add a **folder picker** dropdown/tree below the vault selector.
- Populated by `GET /vaults/:vault_id/folders` when vault is selected.
- Submits `folder_id` alongside the existing upload fields.
- Default: no folder selected = item lands at vault root.

**Fields to add to upload handler:**
```rust
// media-manager/src/upload.rs — UploadRequest
pub folder_id: Option<i32>,
```

### 2. Media list filter (`/media`)

- Add **vault selector** (already in TODO) first.
- Once a vault is selected, show a **folder tree or breadcrumb** to drill down.
- Passes `vault_id` + `folder_id` as query params.
- `folder_id` absent = show all items in vault (flat); present = show items in that folder.

**Fields to add to list/filter:**
```rust
// media-manager/src/list.rs — MediaListQuery
pub vault_id:  Option<String>,
pub folder_id: Option<i32>,

// media-manager/src/models.rs — MediaFilterOptions
pub vault_id:  Option<String>,
pub folder_id: Option<i32>,
```

---

## Touch Points Summary

| File | Change |
|---|---|
| `migrations/014_vault_folders.sql` | New — vault_folders table + folder_id on media_items |
| `crates/vault-manager/src/folders.rs` | New — VaultFolder model + CRUD queries |
| `crates/vault-manager/src/lib.rs` | Wire up folder routes |
| `crates/media-manager/src/upload.rs` | Accept optional `folder_id` on upload |
| `crates/media-manager/src/list.rs` — `MediaListQuery` | Add `vault_id`, `folder_id` |
| `crates/media-manager/src/models.rs` — `MediaFilterOptions` | Same |
| `crates/media-manager/src/search.rs` | Add `AND vault_id = ?` / `AND folder_id = ?` WHERE branches |
| `crates/media-manager/src/list.rs` — `list_media_html/json` | Pass new fields through to filter |
| Upload template | Folder picker (populated from vault selection) |
| Media list template | Vault selector + folder breadcrumb/tree |

---

## Effort Estimate

| Layer | Effort |
|---|---|
| DB migration | Low — 1 table + 1 column |
| Rust model + CRUD (folders.rs) | Moderate |
| Upload handler — accept folder_id | Low |
| Media list filter — vault + folder params | Low |
| Upload template — folder picker UI | Low–moderate |
| Media list template — vault selector + folder nav | Moderate |
| Access control — respect folder group/code | Moderate |
| Risk of breaking existing functionality | Very low — entirely additive |
# Workspace File & Folder Operations

> Audience: developers maintaining workspace-manager crate.

The workspace browser supports move, copy, duplicate, and delete for both files and folders. When a folder has a **typed configuration** in `workspace.yaml`, these operations must keep the metadata in sync.

---

## workspace.yaml folder metadata

Typed folders (e.g. `course`, `static-site`, `web-app`, `agent-collection`) are registered in `workspace.yaml` under the `folders` map, keyed by relative path:

```yaml
name: My Workspace
description: ""
folders:
  my-course:
    type: course
    description: "Intro to Rust"
    metadata:
      ai_provider: openai
  my-course/assets:
    type: default
```

The `FolderConfig` struct holds `folder_type`, `description`, and a free-form `metadata` HashMap. Default-type folders are **not** persisted (inserting `default` removes the entry).

---

## Operations and metadata handling

### Move (same workspace)

- Filesystem: `std::fs::rename`
- Metadata: `WorkspaceConfig::rename_folder_prefix(old, new)` updates the folder key and all sub-folder keys that start with `old/`.

### Move (cross-workspace)

- Filesystem: recursive copy to target workspace, then delete source.
- Metadata:
  1. Load source `WorkspaceConfig`, collect all entries matching the folder prefix.
  2. Insert them into the target `WorkspaceConfig` with adjusted paths.
  3. Remove the entries from the source config.
  4. Save both configs.

### Copy (same workspace)

- Filesystem: recursive directory copy via `copy_dir_recursive`.
- Metadata: `WorkspaceConfig::copy_folder_prefix(from, to)` duplicates all matching entries with new path keys.

### Copy (cross-workspace)

- Filesystem: recursive copy to target workspace directory.
- Metadata:
  1. Load source `WorkspaceConfig`, iterate entries matching the folder prefix.
  2. Insert into target `WorkspaceConfig` with adjusted paths.
  3. Source config is unchanged.
  4. Save target config.

### Duplicate (files and folders)

- Appends `_1` to the name (before extension for files).
- For folders: uses the same copy path, so `copy_folder_prefix` handles metadata.

### Delete

- Filesystem: `std::fs::remove_dir_all` or `std::fs::remove_file`.
- Metadata: `WorkspaceConfig::remove_folder(path)` removes the entry. Sub-folder entries are cleaned up by `sync_with_filesystem` on next load, or explicitly removed if needed.

---

## WorkspaceConfig methods

| Method | Purpose |
|--------|---------|
| `upsert_folder(path, type)` | Add or update; removes entry if type is `default` |
| `rename_folder_prefix(old, new)` | Rename folder + all sub-folder keys |
| `copy_folder_prefix(from, to)` | Clone folder + sub-folder entries to new prefix |
| `remove_folder(path)` | Remove single entry |
| `sync_with_filesystem(root)` | Prune entries for folders that no longer exist on disk |

All methods are in `crates/workspace-manager/src/workspace_config.rs`.

---

## Handler integration

File operation handlers live in `crates/workspace-manager/src/file_ops.rs`:

- `rename_file` — handles both file and folder moves (same + cross-workspace)
- `copy_file` — handles both file and folder copies (same + cross-workspace)
- `delete_file` — handles deletion with metadata cleanup

Each handler loads the workspace config, performs the metadata operation, and saves the config before returning the HTTP response.

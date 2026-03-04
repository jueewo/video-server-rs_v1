# Apps System Implementation Progress

## Goal

Build a generic "JS Tool Viewer" app that serves self-contained HTML/JS teaching
visualizations (and pre-built Preact/Vue3 apps) from workspace folders, and extend
the folder type system to declare which apps can open a given folder type.

## Architecture

```
Folder Type Registry (YAML files)
  └── FolderTypeDefinition.apps: Vec<AppLink>   ← NEW: links type → apps

App examples:
  course      → course-viewer app
  bpmn-sim    → bpmn-viewer app
  js-tool     → js-tool-viewer app  ← NEW crate

JS Tool structure in workspace:
  storage/workspaces/{ws_id}/teaching/       ← folder type: js-tool
    simulated-annealing/
      index.html                             ← drop any self-contained HTML/JS here
      meta.yaml                             ← optional: title, description
    fourier-transform/
      index.html
    my-preact-app/
      index.html                             ← pre-built Preact/Vue3 dist/ works too
      assets/main-abc.js
```

## Routes Added

| Route | Handler | Notes |
|---|---|---|
| `GET /js-apps` | gallery | Lists all js-tool folders across user workspaces |
| `GET /js-apps/{workspace_id}/{*path}` | serve_file | Serves any static file from workspace |

## Files Changed

### Modified
- `crates/workspace-manager/src/folder_type_registry.rs` — `AppLink` struct + `apps` field on `FolderTypeDefinition`
- `crates/workspace-manager/src/lib.rs` — export `AppLink`
- `crates/workspace-manager/src/builtin_types/course.yaml` — added `apps:` section
- `crates/workspace-manager/src/builtin_types/bpmn-simulator.yaml` — added `apps:` section
- `crates/workspace-manager/templates/folder-types/index.html` — show app badges in UI
- `templates/apps.html` — added JS Tool Viewer card
- `Cargo.toml` (root) — added workspace member + dependency
- `src/main.rs` — import, state init, `.merge()`, `app_count: 5`

### Created
- `crates/workspace-manager/src/builtin_types/js-tool.yaml` — new builtin folder type
- `crates/standalone/js-tool-viewer/Cargo.toml`
- `crates/standalone/js-tool-viewer/src/lib.rs`
- `crates/standalone/js-tool-viewer/templates/js_tool/gallery.html`
- `docs/status/APPS_PROGRESS.md` (this file)

## AppLink Schema

```yaml
# In a folder type YAML:
apps:
  - app_id: js-tool-viewer
    label: Open Tools
    icon: play
    url_template: /js-apps/{workspace_id}/{folder_path}
```

`url_template` placeholders:
- `{workspace_id}` — workspace ID from DB
- `{folder_path}` — folder path within workspace (e.g. `teaching`)
- `{slug}` — media item slug (for course/bpmn types that link to media items)

## Security Notes

- File serve handler: `canonicalize()` + prefix check prevents path traversal
- Auth: session required for both gallery and file serve
- Scope: only workspace owner can access their js-tool folders (for now)

## Future Work

- App registry as a separate YAML registry (similar to folder-type-registry)
- Workspace UI: show "Open App" buttons on typed folders (using `apps[]` links)
- Public access to js-tools via access codes
- Multiple apps per folder type shown as separate action buttons
- Admin can install/disable apps from the /apps page

## Status

- [x] AppLink struct + `apps` field on FolderTypeDefinition
- [x] `js-tool.yaml` builtin folder type
- [x] Updated `course.yaml` and `bpmn-simulator.yaml` with app links
- [x] `js-tool-viewer` standalone crate
- [x] Wired into main.rs
- [x] `/apps` page card added
- [x] Folder types UI shows app badges

# Extensibility Architecture

How to add new capabilities without touching `main.rs`.

---

## Two extension points

| What you're adding | Extension point | Facade crate |
|---|---|---|
| A new **workspace folder-type** (renderer) | `workspace-renderers` | always compiled |
| A new **embedded app** (routes + UI) | `workspace-apps` | `apps` feature flag |

---

## Workspace folder-type renderers (`workspace-renderers`)

Each folder type in the workspace browser has a renderer — a crate that implements
`FolderTypeRenderer` from `workspace-core` and owns the content area when that folder
is opened.

### Currently registered renderers

| Crate | `type_id` | Folder type |
|---|---|---|
| `bpmn-viewer` | `"bpmn-simulator"` | BPMN diagram viewer |
| `media-viewer` | `"media-server"` | Media grid inside a workspace folder |
| `course` | `"course"` | Course outline + lesson view |

All three are registered in `crates/workspace-renderers/src/lib.rs` via `register_all()`.
`main.rs` calls this once and never changes.

### Adding a new folder-type renderer

1. **Create the crate** (e.g. `crates/my-feature/`) and implement `FolderTypeRenderer`:

   ```rust
   // crates/my-feature/src/lib.rs
   use workspace_core::{FolderTypeRenderer, FolderViewContext};

   pub struct MyFeatureRenderer { /* pool, storage, etc. */ }

   #[async_trait]
   impl FolderTypeRenderer for MyFeatureRenderer {
       fn type_id(&self) -> &str { "my-feature" }
       async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
           // render HTML content area
       }
   }
   ```

2. **Add to `workspace-renderers/Cargo.toml`**:

   ```toml
   my-feature = { path = "../my-feature" }
   ```

3. **Register in `workspace-renderers/src/lib.rs`**:

   ```rust
   use my_feature::MyFeatureRenderer;

   pub fn register_all(state: &mut WorkspaceManagerState, pool: SqlitePool, user_storage: UserStorageManager) {
       // ... existing registrations ...
       state.register_renderer(Arc::new(MyFeatureRenderer { pool, /* ... */ }));
   }
   ```

4. **Add the folder-type YAML** to `storage/folder-type-registry/my-feature.yaml`.

`main.rs` requires no changes.

### Relationship to `workspace-processors/*`

The `crates/workspace-processors/` crates (`static-site`, `bpmn-simulator`,
`agent-collection`, `course-processor`) are **processing utility libraries** — data
structures, config parsers, build logic. They are not renderers themselves. When a
processor crate gets a workspace viewer, its renderer goes in a crate under `crates/`
and gets registered here.

---

## Workspace apps (`workspace-apps`)

Apps are the embedded route groups that add full UI sections to the server:
`publications` (`/pub/`, `/my-publications`, `/catalog`), `js-tool-viewer` (`/js-apps/`), and `gallery3d`
(`/3d-gallery/`).

Unlike renderers, apps are **optional** — they are compiled only when the `apps`
feature is enabled (included in `full`, the default).

```bash
cargo build                                         # full — all apps included
cargo build --no-default-features --features media  # media-only, no apps compiled
```

### Feature flag topology

```
default = ["full"]

full    = ["media", "course", "bpmn", "apps"]
media   = [video-manager, media-manager, media-viewer]
course  = [course]
bpmn    = [bpmn-viewer]
apps    = [workspace-apps]
           └── publications
           └── js-tool-viewer
           └── gallery3d
```

### Adding a new embedded app

1. **Create the crate** under `crates/standalone/my-app/` with its own routes.

2. **Add to `workspace-apps/Cargo.toml`**:

   ```toml
   my-app = { path = "../standalone/my-app" }
   ```

3. **Merge its router in `workspace-apps/src/lib.rs`**:

   ```rust
   use my_app::{my_app_routes, MyAppState};

   pub fn workspace_app_routes(pool: SqlitePool, storage_base: PathBuf) -> Router {
       // ... existing merges ...
       .merge(my_app_routes(Arc::new(MyAppState { pool, storage_base })))
   }
   ```

`main.rs` requires no changes.

---

## Why this structure

Feature flags answer: *"which deployment topology?"* — a small, stable set.
Renderer registration answers: *"which folder types are available?"* — an open-ended,
growing list.

Using feature flags for individual renderers would mean 20 flags for 20 renderers,
making operator configuration fragile and defaults useless. The facade crate absorbs
growth: `main.rs` is stable regardless of how many renderers or apps exist.

The `workspace-renderers` renderers are all lightweight (Askama templates + simple
queries, no heavy external deps) so always compiling them is correct. The `apps`
feature flag exists because apps add meaningful route surface area and could be
excluded for a headless/API-only deployment.

# Dual-Use App Pattern

> Every functional crate can run in two modes from the same code:
> **embedded** (inline view inside the workspace browser) and
> **standalone** (its own deployable URL with a shell).

---

## Why Dual-Use

The workspace browser is the primary interface for authenticated users. But the
same content needs to be accessible to external clients — satellite apps, customers,
partners — without requiring an account. The dual-use pattern solves this without
duplicating logic or templates.

One crate. Two modes. One line in `main.rs` for each.

---

## Crate Structure

```
crates/my-app/
  src/
    lib.rs           ← FolderTypeRenderer impl + standalone Router
    structure.rs     ← domain logic (read files, build data model)
    render.rs        ← content rendering (markdown, etc.)
  templates/
    my-app/
      folder.html    ← embedded view (workspace chrome provides the outer shell)
      viewer.html    ← standalone view (full page with own header/sidebar/nav)
  askama.toml        ← dirs = ["templates", "../../templates"]
  Cargo.toml
```

The `askama.toml` allows templates to extend `base-tailwind.html` from the root
`templates/` directory — required for both modes.

---

## Embedded Mode — FolderTypeRenderer

```rust
pub struct MyAppRenderer { /* long-lived state: pool, storage */ }

#[async_trait]
impl FolderTypeRenderer for MyAppRenderer {
    fn type_id(&self) -> &str { "my-app" }   // matches registry YAML

    async fn render_folder_view(&self, ctx: FolderViewContext) -> Result<Response, StatusCode> {
        // ctx.workspace_root.join(&ctx.folder_path) = absolute folder path on disk
        // Read files directly — no HTTP, no code needed (user is authenticated)
        let data = structure::load(&ctx.workspace_root.join(&ctx.folder_path))?;
        let tmpl = FolderTemplate { authenticated: true, data, ... };
        Ok(Html(tmpl.render()?).into_response())
    }

    fn extra_routes(&self) -> Option<Router> {
        // Optional: API routes the embedded view needs (e.g. GET /api/my-app/...)
        None
    }
}
```

Register in `main.rs`:
```rust
workspace_state.register_renderer(Arc::new(MyAppRenderer { pool: pool.clone(), storage: user_storage.clone() }));
```

The workspace browser calls `render_folder_view` when the user opens a folder
whose `folder-type` matches `type_id()`. The returned HTML is embedded inside
the workspace chrome — do **not** include a full `<html>` page; use
`{% extends "base-tailwind.html" %}` and the shell provides the outer frame.

---

## Standalone Mode — Public Router

```rust
pub fn my_app_routes(state: Arc<MyAppState>) -> Router {
    Router::new()
        .route("/my-app", get(handler))
        .with_state(state)
}

async fn handler(Query(q): Query<AppQuery>, State(state): ...) -> impl IntoResponse {
    // 1. Look up code → workspace_id + folder_path via workspace_access_codes
    // 2. Read files from disk using storage.workspace_root(workspace_id)
    // 3. Render full-page template (viewer.html with own header/sidebar)
}
```

Register in `main.rs`:
```rust
.merge(my_app_routes(Arc::new(MyAppState { pool: pool.clone(), storage: user_storage.clone() })))
```

The standalone viewer is served by the platform at `/my-app?code={code}`.
No session required — the access code is the credential.

---

## Access Pattern

| User | Path | Auth |
|---|---|---|
| Workspace owner | Opens folder in browser → `render_folder_view` | Session cookie |
| External user / satellite app | `GET /my-app?code={code}` | Access code only |

The standalone handler resolves the code via `workspace_access_codes`:
```sql
SELECT f.workspace_id, f.folder_path
FROM workspace_access_codes wac
JOIN workspace_access_code_folders f ON f.workspace_access_code_id = wac.id
WHERE wac.code = ? AND wac.is_active = 1
  AND (wac.expires_at IS NULL OR wac.expires_at > datetime('now'))
LIMIT 1
```

Then reads files from `storage.workspace_root(workspace_id).join(folder_path)` —
same filesystem access as the embedded mode.

---

## Folder Type Registry

Every embedded renderer needs a YAML file in `storage/folder-type-registry/`:

```yaml
# storage/folder-type-registry/my-app.yaml
id: my-app
name: My App
icon: some-lucide-icon
description: What this folder type does
color: "#6366f1"
builtin: true
```

The `id` must match `FolderTypeRenderer::type_id()`.

---

## Template Strategy

- `folder.html` — extends `base-tailwind.html`, renders inside workspace chrome.
  Provides breadcrumb back to workspace browser. Has Edit links per item.
- `viewer.html` — extends `base-tailwind.html`, renders as full standalone page.
  Has own header (title, description), sidebar navigation, content area.
  No edit links (external user has no session).

Shared styling: use `markdown-body` CSS class (defined inline via `{% block extra_head %}`)
for any rendered prose content — avoids dependency on Tailwind Typography plugin.

---

## Implemented Dual-Use Crates

| Crate | Folder Type | Standalone URL | Status |
|---|---|---|---|
| `crates/bpmn-viewer` | `bpmn-simulator` | — (standalone only via bpmn-js) | Embedded only |
| `crates/media-manager` | `media-server` | — | Embedded only |
| `crates/course` | `course` | `GET /course?code=` | **Full dual-use** |

---

## See Also

- `workspace-core/src/lib.rs` — `FolderTypeRenderer` trait definition and docs
- `crates/course/` — reference implementation (first full dual-use crate)
- `docs/apps/course-viewer.md` — course viewer user and developer guide
- `docs/management/WORKSPACE_ACCESS_CODES.md` — access code API used by standalone mode

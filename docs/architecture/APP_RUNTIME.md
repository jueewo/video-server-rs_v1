# App Runtime — Sidecar Architecture

## Overview

The `app-runtime` crate enables full-stack workspace apps by spawning backend processes on demand and proxying API requests to them. Backends can be **Bun/TypeScript**, **compiled Rust binaries**, or **any executable** that speaks HTTP.

## App Capability Tiers

| Tier | Frontend | Data | Backend | Example |
|------|----------|------|---------|---------|
| 1 | HTML/CSS/JS | None | None | Calculator, diagram editor |
| 2 | HTML/CSS/JS | Bundled .db | Client-side (sql.js WASM) | Data dashboard |
| 3 | HTML/CSS/JS | Bundled .db | Platform REST API | *(planned)* |
| **4** | **HTML/CSS/JS** | **App-managed** | **Sidecar process** | **Task tracker, Rust micro API** |

## Architecture

```
Browser → Platform (Rust/axum) ─┬─ Static files (index.html, CSS, JS)
                                │   served by js-tool-viewer (existing)
                                │
                                └─ /api/apps/{workspace_id}/{folder_path}/*
                                    → Proxy → Sidecar on localhost:{port}
                                               ├── server.ts (Bun)
                                               ├── ./micro-server (Rust)
                                               ├── or any executable
                                               └── data.db (optional)
```

## App Structure

### Bun app (default)
```
my-app/
  meta.yaml          # title + description
  index.html          # frontend (served by js-tool-viewer)
  server.ts           # Bun backend (spawned by app-runtime)
  data.db             # SQLite database (optional)
  package.json        # npm dependencies (optional, auto-installed)
```

### Custom binary app
```
my-rust-app/
  meta.yaml          # must include server_command
  index.html          # frontend
  micro-server        # compiled binary (Rust, Go, etc.)
  data.db             # SQLite database (optional)
```

**Detection:** The proxy walks the folder path looking for `server.ts`, `server.js`, or a `meta.yaml` with `server_command`. The first match becomes the app root.

## Sidecar Contract

Every sidecar backend — regardless of language — must implement:

1. **Accept `--port=NNNN`** as a CLI argument (platform assigns a random port)
2. **Expose `GET /health`** returning HTTP 200 (platform polls this for readiness)

That's it. Everything else is up to the app.

### Bun example
```typescript
const port = parseInt(process.argv.find(a => a.startsWith("--port="))?.split("=")[1] ?? "3001");
Bun.serve({
  port,
  async fetch(req) {
    if (new URL(req.url).pathname === "/health") return new Response("ok");
    // ... app routes
  }
});
```

### Rust/axum example
```rust
fn parse_port() -> u16 {
    std::env::args()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.strip_prefix("--port=").unwrap().parse().ok())
        .unwrap_or(3001)
}
// Router includes: .route("/health", get(|| async { "ok" }))
```

## Custom Server Command

Add `server_command` to `meta.yaml` to use any executable:

```yaml
title: My Rust App
server_command: ./micro-server
```

The platform splits the command on whitespace, appends `--port=NNNN`, and spawns the process. Examples:

```yaml
server_command: ./micro-server             # compiled Rust binary
server_command: python server.py           # Python
server_command: ./server                   # Go binary
server_command: deno run --allow-net server.ts  # Deno
```

When `server_command` is set, `server.ts`/`server.js` and Bun are not required.

## Request Flow

1. User browses to `/js-apps/{workspace_id}/{folder}/` → js-tool-viewer serves `index.html`
2. Frontend JS calls `/api/apps/{workspace_id}/{folder}/api/tasks`
3. `app-runtime` proxy handler receives the request
4. `SidecarManager::ensure_running()` checks if a process is already running for this app
   - If not: picks a random port, spawns the backend, waits for `/health`
   - If yes: returns existing port, updates `last_request` timestamp
5. Proxy forwards the request to `http://127.0.0.1:{port}/api/tasks`
6. Sidecar response is forwarded back to the browser

## Process Lifecycle

The platform fully manages each sidecar's lifecycle — **no manual start/stop needed**.

| Event | What happens |
|-------|-------------|
| **First API request** | Process spawned, health-checked, then request proxied |
| **Subsequent requests** | Routed to existing process, idle timer reset |
| **5 minutes idle** | Background task kills the process (SIGKILL) |
| **Next request after idle stop** | Fresh process spawned transparently |
| **Sidecar crashes** | Detected via `try_wait()`, respawned on next request |
| **Platform shutdown** | `Drop` impl kills all sidecars immediately |
| **Binary updated** | Kill old sidecar (or wait for idle timeout), next request spawns new version |

- **Health check:** Polls `GET /health` every 100ms for up to 10s after spawn
- **Idle sweep:** Background task runs every 60s, kills sidecars idle >5 min
- **Auto-install:** For Bun apps, if `package.json` exists without `node_modules/`, runs `bun install` before first spawn

## Key Components

### `SidecarManager` (`crates/app-runtime/src/sidecar.rs`)

```rust
pub struct SidecarManager {
    sidecars: DashMap<String, AppSidecar>,  // key: "{workspace_id}/{folder}"
    http: reqwest::Client,
}
```

Methods:
- `ensure_running(workspace_id, folder, app_dir) → Result<u16>` — returns port
- `cleanup_idle(max_idle: Duration)` — stops idle sidecars
- `stop(workspace_id, folder)` — stops a specific sidecar
- `list_active() → Vec<(String, u16)>` — for debugging

Reads `meta.yaml` for `server_command`. Falls back to `bun run server.ts` / `bun run server.js`.

### `proxy_handler` (`crates/app-runtime/src/proxy.rs`)

- Route: `/api/apps/{workspace_id}/{*rest}`
- Splits `rest` into app folder path + API path by finding `server.ts`/`server.js` or `server_command` in `meta.yaml`
- Forwards all HTTP methods, headers, and body
- Skips hop-by-hop headers (host, connection, transfer-encoding)
- Body limit: 10 MB

### `db_persist` (`crates/app-runtime/src/db_persist.rs`)

Optional persistence for client-side sql.js (WASM) apps:
- `POST /api/app-db/{workspace_id}/{*folder_path}` — save the DB blob (validates SQLite header, atomic write)
- `GET /api/app-db/{workspace_id}/{*folder_path}` — check writable status
- Enabled per-app via `db_writable: true` in `meta.yaml`

## Port Management

- `TcpListener::bind("127.0.0.1:0")` finds a random available port
- Port passed via `--port={port}` CLI argument
- Sidecar only listens on localhost (not externally reachable)

## Micro Server — Stripped-Down Rust Binary

The `crates/standalone/micro-server` crate is a minimal axum template for custom sidecar apps. It demonstrates:

- Using the `media-core` crate for file type detection
- A SQLite notes CRUD API
- The full sidecar contract (`--port` + `/health`)

```
cargo build --package micro-server --release
# → target/release/micro-server (4.8 MB with media-core + sqlx)
```

Compare: the full platform binary is ~130 MB. The micro-server pulls in only what it needs.

To deploy: copy the binary + `index.html` + `meta.yaml` into a workspace folder.

### `pub_app_proxy` (`crates/workspace-apps/src/pub_app_proxy.rs`)

Proxy for published runtime apps:
- Route: `/api/pub-apps/{slug}/{*rest}`
- Looks up the publication by slug → resolves workspace_id + folder_path
- Walks `rest` segments to find the app root (same logic as `find_app_root`)
- Delegates to `forward_to_sidecar()` — shares the same `SidecarManager` as the main proxy
- Published apps and workspace apps use the same sidecar instances

## Frontend API URL Convention

The frontend derives its API base URL from its page URL, handling both workspace and published contexts:

```javascript
const pagePath = location.pathname.replace(/\/index\.html$/, '').replace(/\/$/, '');
const API = pagePath.startsWith('/pub/')
  ? pagePath.replace(/^\/pub\//, '/api/pub-apps/')    // published app
  : pagePath.replace(/^\/js-apps\//, '/api/apps/');    // workspace app
```

| Context | Page URL | API URL |
|---------|----------|---------|
| Workspace | `/js-apps/ws-123/task-tracker/` | `/api/apps/ws-123/task-tracker/` |
| Published | `/pub/task-tracker/` | `/api/pub-apps/task-tracker/` |

## Course Embedding

Runtime apps can be embedded in course lessons:

````markdown
```app-embed height=600
/js-apps/{workspace_id}/{folder}/
```
````

## Folder Types

Three folder types serve different app patterns:

| Type | Name | Icon | Color | Purpose |
|------|------|------|-------|---------|
| `js-tool` | JavaScript Tool Collection | code-2 | Amber | Parent folder containing multiple JS tools as sub-folders |
| `web-app` | Web App | globe | Blue | Single standalone browser app (WASM, sql.js, SPA) |
| `runtime-app` | Runtime Application | server | Purple | Full-stack app with sidecar backend (Bun or custom binary) |

Defined in `crates/workspace-manager/src/builtin_types/`.

### Single-app serving

When `js-tool-viewer` opens a folder that has `index.html` at its root (web-app or runtime-app), it serves the app directly. For collection folders (`js-tool`), it scans sub-directories for tools.

The viewer redirects to a trailing-slash URL (`/js-apps/{ws}/{folder}/`) so that relative asset fetches (`data.db`, CSS, JS) resolve correctly.

## Security Notes

- Sidecar runs as the same OS user as the platform (no sandboxing)
- Each sidecar's CWD is set to its app folder
- Only listens on `127.0.0.1` (not externally reachable)
- API proxy requires the workspace folder to exist on disk
- No auth check on the proxy endpoint yet (future: add workspace ownership verification)

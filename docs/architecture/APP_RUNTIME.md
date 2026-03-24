# App Runtime — Bun Sidecar Architecture

## Overview

The `app-runtime` crate enables full-stack workspace apps by spawning Bun processes on demand and proxying API requests to them. This extends the platform beyond static HTML/JS apps to support server-side logic, database writes, external API calls, and more.

## App Capability Tiers

| Tier | Frontend | Data | Backend | Example |
|------|----------|------|---------|---------|
| 1 | HTML/CSS/JS | None | None | Calculator, diagram editor |
| 2 | HTML/CSS/JS | Bundled .db | Client-side (sql.js WASM) | Data dashboard |
| 3 | HTML/CSS/JS | Bundled .db | Platform REST API | *(planned)* |
| **4** | **HTML/CSS/JS** | **App-managed** | **Bun sidecar** | **Task tracker, CRM** |

## Architecture

```
Browser → Platform (Rust/axum) ─┬─ Static files (index.html, CSS, JS)
                                │   served by js-tool-viewer (existing)
                                │
                                └─ /api/apps/{workspace_id}/{folder_path}/*
                                    → Proxy → Bun sidecar on localhost:{port}
                                               ├── server.ts
                                               ├── data.db (bun:sqlite)
                                               └── (any backend logic)
```

## App Structure

```
my-app/
  meta.yaml          # title + description (for js-tool-viewer gallery)
  index.html          # frontend (served by platform via js-tool-viewer)
  server.ts           # backend (spawned as Bun process by app-runtime)
  data.db             # SQLite database (read/write by server.ts)
  package.json        # optional npm dependencies
```

**Detection:** The proxy handler walks the folder path looking for `server.ts` or `server.js`. If found, that directory becomes the app root.

## Request Flow

1. User browses to `/js-apps/{workspace_id}/{folder}/` → js-tool-viewer serves `index.html`
2. Frontend JS calls `/api/apps/{workspace_id}/{folder}/api/tasks`
3. `app-runtime` proxy handler receives the request
4. `SidecarManager::ensure_running()` checks if Bun is already running for this app
   - If not: picks a random port, spawns `bun run server.ts --port={port}`, waits for `/health` to respond
   - If yes: returns existing port, updates `last_request` timestamp
5. Proxy forwards the request to `http://127.0.0.1:{port}/api/tasks`
6. Sidecar response is forwarded back to the browser

## Process Lifecycle

- **Spawn on demand:** First API request triggers process creation
- **Health check:** Polls `GET /health` every 100ms for up to 10s after spawn
- **Idle timeout:** Background task sweeps every 60s, kills sidecars idle >5 minutes
- **Crash recovery:** `try_wait()` detects exited processes, auto-respawns on next request
- **Shutdown cleanup:** `Drop` implementation kills all sidecars when the platform stops
- **Auto-install:** If `package.json` exists without `node_modules/`, runs `bun install` before first spawn

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

### `proxy_handler` (`crates/app-runtime/src/proxy.rs`)

- Route: `/api/apps/{workspace_id}/{*rest}`
- Splits `rest` into app folder path + API path by finding `server.ts`/`server.js`
- Forwards all HTTP methods, headers, and body
- Skips hop-by-hop headers (host, connection, transfer-encoding)
- Body limit: 10 MB

### Routes (`crates/app-runtime/src/lib.rs`)

- Single route handles GET, POST, PUT, DELETE, PATCH
- Background cleanup task spawned on startup
- Wired into `src/main.rs` with ~3 lines

## Port Management

- `TcpListener::bind("127.0.0.1:0")` finds a random available port
- Port passed to Bun via `--port={port}` CLI argument
- Sidecar only listens on localhost (not externally reachable)

## Server.ts Contract

Apps must implement:

```typescript
// Required: health check endpoint
"/health" → Response 200

// Required: parse port from CLI args
const port = parseInt(process.argv.find(a => a.startsWith("--port="))?.split("=")[1] ?? "3001");
```

Everything else is up to the app. Example using `bun:sqlite`:

```typescript
import { Database } from "bun:sqlite";
const db = new Database("data.db");

Bun.serve({
  port,
  async fetch(req) {
    // Handle routes...
  }
});
```

## Frontend API URL Convention

The frontend derives its API base URL by replacing the js-tool-viewer path prefix:

```javascript
const pagePath = location.pathname.replace(/\/index\.html$/, '').replace(/\/$/, '');
const API = pagePath.replace(/^\/js-apps\//, '/api/apps/');
// /js-apps/ws-123/demo-apps/my-app → /api/apps/ws-123/demo-apps/my-app
```

## Course Embedding

Runtime apps can be embedded in course lessons using the existing `app-embed` block:

````markdown
```app-embed height=600
/js-apps/{workspace_id}/{folder}/
```
````

The app renders in an iframe. API calls work because the frontend uses relative URL derivation from its own location.

## Folder Type

Registered as `runtime-app` in `crates/workspace-manager/src/builtin_types/runtime-app.yaml`. Opens via js-tool-viewer (same as `js-tool` type).

## Security Notes

- Sidecar runs as the same OS user as the platform (no sandboxing)
- Each sidecar's CWD is set to its app folder
- Only listens on `127.0.0.1` (not externally reachable)
- API proxy requires the workspace folder to exist on disk
- No auth check on the proxy endpoint yet (future: add workspace ownership verification)

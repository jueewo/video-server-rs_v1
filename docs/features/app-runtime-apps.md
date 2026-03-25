# Runtime Apps — Full-Stack Apps with Sidecar Backend

## What Are Runtime Apps?

Runtime apps are workspace apps that include a server-side backend. The platform spawns the backend process on demand, proxies API requests to it, and manages its lifecycle automatically. Unlike static JS tools (which run entirely in the browser), runtime apps can:

- **Read and write data** to a SQLite database (persistent, server-side)
- **Call external APIs** (Stripe, OpenAI, SMTP, webhooks, etc.)
- **Process files** server-side (PDF generation, image manipulation)
- **Run custom business logic** in any language (TypeScript, Rust, Go, Python…)

## Quick Start — Bun Backend

1. Create a folder in your workspace with three files:

**`meta.yaml`**
```yaml
title: My App
description: A full-stack app with Bun backend.
```

**`server.ts`**
```typescript
import { Database } from "bun:sqlite";

const port = parseInt(process.argv.find(a => a.startsWith("--port="))?.split("=")[1] ?? "3001");
const db = new Database("data.db");
db.run("CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY, name TEXT)");

Bun.serve({
  port,
  async fetch(req) {
    const url = new URL(req.url);
    if (url.pathname === "/health") return new Response("ok");
    if (url.pathname === "/api/items" && req.method === "GET") {
      return Response.json(db.query("SELECT * FROM items").all());
    }
    if (url.pathname === "/api/items" && req.method === "POST") {
      const { name } = await req.json();
      db.run("INSERT INTO items (name) VALUES (?)", [name]);
      return Response.json({ ok: true });
    }
    return Response.json({ error: "not found" }, { status: 404 });
  },
});
```

**`index.html`**
```html
<!DOCTYPE html>
<html>
<body>
  <h1>My App</h1>
  <script>
    const API = location.pathname
      .replace(/\/index\.html$/, '').replace(/\/$/, '')
      .replace(/^\/js-apps\//, '/api/apps/');

    fetch(API + '/api/items').then(r => r.json()).then(console.log);
  </script>
</body>
</html>
```

2. Set the folder type to `runtime-app` (or `js-tool`) in workspace settings
3. Open the app — the backend starts automatically on first API call

## Quick Start — Compiled Rust Binary

1. Build a binary from `crates/standalone/micro-server` (or your own):
   ```bash
   cargo build --package micro-server --release
   ```

2. Copy into a workspace folder:
   ```
   my-rust-app/
     meta.yaml
     index.html
     micro-server      ← the compiled binary (4.8 MB)
   ```

3. **`meta.yaml`** — point to the binary:
   ```yaml
   title: My Rust API
   server_command: ./micro-server
   ```

4. Open the app — the platform spawns the binary on the first API call

## How It Works

- The **frontend** (`index.html`) is served by the existing js-tool-viewer
- The **backend** is spawned as a process by the platform on the first API request
- API calls from the frontend go to `/api/apps/{workspace_id}/{folder}/...` and are proxied to the backend
- The backend **auto-stops** after 5 minutes of inactivity and **auto-restarts** on the next request
- If a Bun app has `package.json`, `bun install` runs automatically before first start

## Sidecar Contract

Every backend — regardless of language — must:

1. **Accept `--port=NNNN`** as a CLI argument
2. **Expose `GET /health`** returning HTTP 200

That's it. The platform handles everything else.

## Process Lifecycle

You never need to manually start or stop a sidecar. The platform manages the full lifecycle:

| Event | What happens |
|-------|-------------|
| First API request | Process spawned, health-checked, request proxied |
| Subsequent requests | Routed to existing process, idle timer reset |
| 5 minutes idle | Background task kills the process |
| Next request after stop | Fresh process spawned transparently |
| Sidecar crashes | Detected automatically, respawned on next request |
| Platform shutdown | All sidecars killed immediately |
| Binary updated on disk | Kill old sidecar or wait for idle timeout; next request spawns new version |

## Custom Server Binaries

Any executable can run as a sidecar. Add `server_command` to `meta.yaml`:

```yaml
title: My Custom App
server_command: ./my-server
```

The platform splits the command on whitespace, appends `--port=NNNN`, and spawns it. Examples:

```yaml
# Compiled Rust binary
server_command: ./micro-server

# Python
server_command: python server.py

# Go
server_command: ./server

# Deno
server_command: deno run --allow-net server.ts
```

When `server_command` is set, `server.ts`/`server.js` and Bun are not required.

## Micro Server — Stripped-Down Rust Template

The `crates/standalone/micro-server` crate is a minimal axum binary template. It includes:

- **File Inspector** — uses the platform's `media-core` crate for MIME type detection
- **Notes API** — SQLite CRUD, persisted to `data.db`
- Full sidecar contract (`--port` + `/health`)

| | Full platform | Micro server |
|---|---|---|
| Binary size (release) | ~130 MB | 4.8 MB |
| Crates | 43 | 4 (axum, media-core, sqlx, tower-http) |
| Startup | Seconds (DB, OIDC, etc.) | Instant |

Add any workspace crate as a dependency to build focused microservices:
```toml
[dependencies]
media-core = { path = "../../media-core" }
common = { path = "../../common" }
```

## Requirements

- **Bun** must be installed on the server (`bun` in PATH) — only for Bun-based apps
- Custom binary apps only need the binary in the app folder
- The server must expose `GET /health` returning HTTP 200
- The server must accept `--port=NNNN` as a CLI argument

## Embedding in Courses

Add to any course lesson markdown:

````markdown
```app-embed height=600
/js-apps/{workspace_id}/{folder}/
```
````

The app renders in an iframe with full interactivity.

## Examples

| Example | Type | Location |
|---------|------|----------|
| SQLite Data Dashboard | Tier 2 (WASM) | `examples/sqlite-data-app/` |
| Task Tracker | Tier 4 (Bun) | `examples/task-tracker/` |
| Rust Sidecar | Tier 4 (Rust) | `examples/rust-sidecar/` |

## Vendor Libraries

Runtime app frontends can use any vendored library:
- `/static/vendor/daisyui.min.css` — DaisyUI CSS framework
- `/static/vendor/htmx.min.js` — htmx
- `/static/vendor/alpine.min.js` — Alpine.js
- `/static/vendor/sql-js/` — sql.js (client-side SQLite, for read-only use cases)

## Comparison

| Feature | Static (js-tool) | Runtime — Bun | Runtime — Custom Binary |
|---------|------------------|---------------|------------------------|
| Frontend | HTML/CSS/JS | HTML/CSS/JS | HTML/CSS/JS |
| Data storage | Client-side only | Server-side SQLite | Any |
| External API calls | Browser (CORS) | Server (no CORS) | Server (no CORS) |
| Backend language | None | TypeScript | Any (Rust, Go, Python…) |
| Dependencies | None | Bun | Binary in app folder |
| Auto-start/stop | N/A | Yes (5 min idle) | Yes (5 min idle) |
| Config | — | `server.ts` | `server_command` in meta.yaml |
| Binary size | — | N/A (interpreted) | 1.8–5 MB (Rust release) |

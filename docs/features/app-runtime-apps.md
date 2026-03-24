# Runtime Apps — Full-Stack Apps with Bun Backend

## What Are Runtime Apps?

Runtime apps are workspace apps that include a server-side backend powered by Bun. Unlike static JS tools (which run entirely in the browser), runtime apps can:

- **Read and write data** to a SQLite database (persistent, server-side)
- **Call external APIs** (Stripe, OpenAI, SMTP, webhooks, etc.)
- **Process files** server-side (PDF generation, image manipulation)
- **Run custom business logic** in TypeScript

## Quick Start

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

## How It Works

- The **frontend** (`index.html`) is served by the existing js-tool-viewer
- The **backend** (`server.ts`) is spawned as a Bun process by the platform on the first API request
- API calls from the frontend go to `/api/apps/{workspace_id}/{folder}/...` and are proxied to the Bun process
- The backend **auto-stops** after 5 minutes of inactivity and **auto-restarts** on the next request
- If `package.json` exists, `bun install` runs automatically before first start

## Requirements

- **Bun** must be installed on the server (`bun` in PATH)
- `server.ts` must expose a `/health` endpoint returning HTTP 200
- `server.ts` must accept `--port=NNNN` as a CLI argument

## Embedding in Courses

Add to any course lesson markdown:

````markdown
```app-embed height=600
/js-apps/{workspace_id}/{folder}/
```
````

The app renders in an iframe with full interactivity.

## Example: Task Tracker

See `examples/task-tracker/` for a complete example with:
- Bun backend using `bun:sqlite`
- CRUD API (create, toggle, delete tasks)
- DaisyUI-styled frontend
- Persistent SQLite storage

## Vendor Libraries

Runtime app frontends can use any vendored library:
- `/static/vendor/daisyui.min.css` — DaisyUI CSS framework
- `/static/vendor/htmx.min.js` — htmx
- `/static/vendor/alpine.min.js` — Alpine.js
- `/static/vendor/sql-js/` — sql.js (client-side SQLite, for read-only use cases)

## Comparison: Static vs Runtime Apps

| Feature | Static (js-tool) | Runtime (runtime-app) |
|---------|------------------|----------------------|
| Frontend | HTML/CSS/JS | HTML/CSS/JS |
| Data storage | Client-side only (localStorage, sql.js) | Server-side SQLite (persistent) |
| External API calls | From browser (CORS limited) | From server (no CORS) |
| Custom backend logic | No | Yes (TypeScript) |
| Dependencies | None | Bun on server |
| Auto-start/stop | N/A | Yes (5 min idle timeout) |

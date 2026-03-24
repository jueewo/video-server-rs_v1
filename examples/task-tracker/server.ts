import { Database } from "bun:sqlite";

// Parse port from CLI args: --port=NNNN
const portArg = process.argv.find((a) => a.startsWith("--port="));
const port = parseInt(portArg?.split("=")[1] ?? "3001");

// Open or create the SQLite database in the app's own directory
const db = new Database("data.db");
db.run(`
  CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
  )
`);

Bun.serve({
  port,
  async fetch(req) {
    const url = new URL(req.url);
    const path = url.pathname;

    // Health check for platform readiness probe
    if (path === "/health") {
      return Response.json({ status: "ok" });
    }

    // GET /api/tasks — list all tasks
    if (path === "/api/tasks" && req.method === "GET") {
      const tasks = db.query("SELECT * FROM tasks ORDER BY created_at DESC").all();
      return Response.json(tasks);
    }

    // POST /api/tasks — create a task
    if (path === "/api/tasks" && req.method === "POST") {
      const { title } = (await req.json()) as { title: string };
      if (!title?.trim()) {
        return Response.json({ error: "title is required" }, { status: 400 });
      }
      const result = db.run("INSERT INTO tasks (title) VALUES (?)", [title.trim()]);
      return Response.json({ id: result.lastInsertRowid, ok: true });
    }

    // POST /api/tasks/:id/toggle — toggle done state
    const toggleMatch = path.match(/^\/api\/tasks\/(\d+)\/toggle$/);
    if (toggleMatch && req.method === "POST") {
      const id = parseInt(toggleMatch[1]);
      db.run("UPDATE tasks SET done = NOT done WHERE id = ?", [id]);
      return Response.json({ ok: true });
    }

    // DELETE /api/tasks/:id — delete a task
    const deleteMatch = path.match(/^\/api\/tasks\/(\d+)$/);
    if (deleteMatch && req.method === "DELETE") {
      const id = parseInt(deleteMatch[1]);
      db.run("DELETE FROM tasks WHERE id = ?", [id]);
      return Response.json({ ok: true });
    }

    return Response.json({ error: "not found" }, { status: 404 });
  },
});

console.log(`Task Tracker API running on http://localhost:${port}`);

# Strategic Feedback — Session 3

## Direction: Workspace is the Core — Remodel First

The decision: the workspace is the foundational concept. The media server remodel
is not optional polish — it must be done before building further.

### Why Now

Building more features on top of the current dual-storage model deepens the confusion
and makes the eventual remodel harder. Doing it later means migrating everything twice.

### Order of Operations

1. **Remodel first** — consolidate storage into workspaces, retire vault as a user-facing
   concept, reframe transcoding as a service on workspace files
2. **Then build** — new features, satellite apps, app ecosystem — all on the clean foundation

### Scope of the Remodel

Significant but bounded. The core changes:

- `media_items` table becomes a lightweight **index over workspace files**, not the authoritative record
- Upload goes to workspace, not vault
- Transcoding triggered on workspace files, output written back to workspace
- Serving routes read from workspace storage
- Access/sharing defined at workspace level

What does NOT change: HLS pipeline logic, BPMN viewer, workspace browser, WebDAV server,
folder-type app system. The storage layer and upload flow move — the feature logic stays.

### Sharing Model

Sharing must be possible to allow external tools (satellite apps, AI agents, third parties)
to access content from the platform. The model:

- **Workspace-level access codes** — share a folder or file via a token, no user account needed
- **Public serving routes** — clean URLs any external app can embed or stream from
- **API keys** — programmatic access for satellite apps and automation
- **WebDAV** — filesystem-level access for tools that prefer it
- **MCP** — AI-agent access

No file needs to move or be "published" to become shareable. Sharing is a permission
on the workspace file, not a migration to a different storage system.

### Key Principle

> Maximum consolidation, no bloating of concepts.

One place files live. One way to upload. One way to share. Transcoding and serving
are services applied to those files — not reasons to store them differently.

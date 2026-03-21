# Strategic Feedback — Session 2

## Direction: Does the Media Server Bloat the Concept?

The question: could images and documents just live in workspaces, making the
media vault unnecessary for most file types?

### What the Media Server Genuinely Adds

- **HLS transcoding** — stateful, multi-quality, progress tracking. Complex enough to warrant its own system
- **RTMP live streaming** — completely orthogonal, stays regardless
- **Per-file public sharing via access codes** — hard to replicate at workspace level

### What It Adds That Workspaces Could Handle

- Image storage and serving — a workspace file + a serving route is functionally the same
- PDF serving — already works in the workspace viewer
- Document management — workspaces already do this
- Thumbnails — could be generated on-demand from workspace files

### The Friction the Dual System Creates Today

There is already a "Publish to Vault" flow in the workspace browser — users consciously
move files *from* workspaces *into* the media system. That's a conceptual seam users
have to understand. Why do files live in two places?

### The Cleaner Mental Model

- Everything lives in workspaces
- Transcoding is a **service applied to workspace files** — trigger it, writes HLS output back to the workspace
- Serving is just serving workspace files with the right route
- Sharing is handled by workspace-level access control or access codes on folders

### What Would Actually Be Lost

The `media_items` database table with slugs, metadata, and per-file access codes.
That enables searching across all media regardless of which workspace it's in — not
nothing, but also significant complexity.

### Verdict

- **Images and documents** — yes, probably bloat as a separate system
- **Video transcoding** — the pipeline earns its place, but could be repositioned as a
  *service* that operates on workspace files rather than a separate storage system

The concept would be considerably tighter if the media vault was just
**"the transcoding service"** rather than an alternative place to store files.

### Action Items

- [ ] Evaluate moving image/document storage fully into workspaces
- [ ] Reframe the media vault as a transcoding + streaming service, not a storage layer
- [ ] Replace "Publish to Vault" flow with "Transcode this file" action on workspace files
- [ ] Define workspace-level access codes for folder/file sharing
- [ ] Assess cross-workspace search as a separate concern (index workspace files, not vault)

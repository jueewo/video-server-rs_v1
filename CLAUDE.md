# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A production-ready media management platform with live streaming capabilities built in Rust. The system handles video/image/document uploads, HLS video transcoding, RTMP live streaming via MediaMTX, session-based authentication, access control, and vault-based storage isolation.

## Build & Development Commands

### Building
```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build --package media-manager

# Release build
cargo build --release

# Build main binary
cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Test specific crate
cargo test --package common

# Test with logging
RUST_LOG=debug cargo test
```

### Database Migrations
**Critical**: SQLite migrations are NOT auto-applied. Apply manually:
```bash
sqlite3 media.db < migrations/NNN_description.sql
```

The `_sqlx_migrations` table only tracks one migration; numbered migrations 001-013 were applied outside sqlx. New migrations (014+) must be applied manually.

### Running
```bash
# Start main server (requires MediaMTX for streaming)
cargo run --release

# Start with OpenTelemetry tracing
OTLP_ENDPOINT=http://localhost:4317 cargo run

# Run admin scripts
cargo run --bin generate-thumbnails
cargo run --bin migrate_storage
cargo run --bin cleanup_missing_media
```

### MediaMTX Integration
MediaMTX handles RTMP/HLS streaming. Run separately:
```bash
mediamtx mediamtx.yml
```

## Architecture

### Workspace Structure
Cargo workspace with 23+ crates organized in `crates/`:

**Core Infrastructure:**
- `common` - Shared types, storage manager, database utilities
- `media-core` - Media type detection, EXIF extraction
- `access-control` - Unified access control service (user ownership, groups, access codes)

**Media Management:**
- `media-manager` - **Unified media endpoint** (videos, images, documents)
  - Handles upload, listing, search, CRUD, serving
  - HLS transcoding integration with real-time WebSocket progress
  - Vault-based storage isolation
- `video-manager` - HLS transcoding pipeline (8 stages), FFmpeg integration
- `docs-viewer` - Markdown/BPMN/PDF viewers
- `bpmn-viewer`, `pdf-viewer` - Specialized viewers

**Authentication & Authorization:**
- `user-auth` - OIDC authentication (Casdoor), session management
- `access-codes` - Shareable access codes for media
- `access-groups` - User groups and permissions
- `api-keys` - API key management

**Utilities:**
- `vault-manager` - Storage vault management
- `rate-limiter` - tower_governor integration
- `workspace-manager` - Multi-tenant workspace support

**Standalone Apps:**
- `standalone/3d-gallery` - Three.js 3D gallery viewer
- `standalone/media-mcp` - MCP server for Claude Desktop
- `standalone/course-viewer` - Course content viewer
- `standalone/media-cli` - CLI tool

### Key Architectural Patterns

**Unified Media Table:**
The `media_items` table consolidates all media types (videos, images, documents). Legacy `videos`, `images`, `documents` tables have been dropped (migration 011).

**Vault-Based Storage (nested structure):**
Media and thumbnails use a symmetric nested layout under each vault:
```
storage/vaults/{vault_id}/
  media/
    images/{slug}.webp          # converted WebP (+ {slug}_original.ext if kept)
    images/{slug}.svg           # SVGs stored as-is
    videos/{slug}/video.mp4     # MP4 direct playback
    videos/{slug}/              # HLS output directory (index.m3u8, segments)
    documents/{filename}
  thumbnails/
    images/{slug}_thumb.webp
    videos/{slug}_thumb.webp
    documents/{slug}_thumb.webp
```
Key methods in `common::storage::UserStorageManager`:
- `vault_nested_media_dir(vault_id, media_type)` → `storage/vaults/{vault_id}/media/{type}/`
- `vault_nested_media_path(vault_id, media_type, slug)` → adds `/{slug}` suffix
- `find_media_file(vault_id, media_type, filename)` → returns `Some(path)` if exists
- `vault_thumbnails_dir(vault_id, media_type)` → `storage/vaults/{vault_id}/thumbnails/{type}/`
- `find_thumbnail(vault_id, media_type, slug)` → looks for `{slug}_thumb.webp`

**Do not use** `vault_media_dir` / `vault_media_path` for new code — those point to the old flat path without the `media/` prefix.

**HLS Transcoding Pipeline (video-manager):**
8 stages: Validation → Metadata → Transcoding → Thumbnail → Poster → Database → Cleanup → Completion
- Multi-quality adaptive bitrate (1080p, 720p, 480p, 360p)
- Auto quality selection based on source resolution (no upscaling)
- Progress tracking via DashMap with WebSocket updates

**Access Control Service (access-control crate):**
Centralized authorization checking:
- User ownership validation
- Group membership verification
- Access code validation
- Combines multiple permission sources

**Rate Limiting:**
Three tiers based on resource intensity:
- Default: 60 RPM (most endpoints)
- Upload: 15 RPM (resource-intensive)
- Serving: 300 RPM (high-throughput media delivery)

### Template Rendering (Askama)
- Askama 0.13 (templates no longer implement `IntoResponse`)
- Must render explicitly: `Html(template.render().map_err(...))`
- Route params: `{param}` not `:param` (Axum 0.8)
- Templates in `crates/*/templates/`

### State Management Pattern
Each module defines its own state struct:
```rust
pub struct MediaManagerState {
    pub pool: SqlitePool,
    pub storage_dir: String,
    pub user_storage: UserStorageManager,
    pub access_control: Arc<AccessControlService>,
    pub hls_progress: Arc<ProgressTracker>, // WebSocket progress
}
```

State is passed via `State<T>` extractors in Axum handlers.

## Critical Implementation Details

### FFmpeg Video Transcoding
HLS transcoding uses `video_manager::hls::transcode_to_hls()`:
- Requires FFmpeg and ffprobe in PATH
- Creates multiple quality variants (auto-selected based on source)
- Uses `.output()` not `.status()` to capture stderr for error logging
- Quality selection: only creates variants ≤ source resolution

**Common Error:** "All quality transcoding attempts failed"
- Check FFmpeg stderr in logs
- Verify source video resolution
- Ensure codec support (libx264, aac)

### WebSocket Progress Tracking
Real-time HLS transcoding progress:
1. Upload handler returns 202 ACCEPTED with slug
2. Frontend connects to `/api/media/{slug}/progress/ws`
3. Backend spawns tokio task, updates `ProgressTracker` DashMap
4. WebSocket handler polls every 500ms, sends JSON updates
5. Progress: 10% (upload) → 15% (validate) → 20% (metadata) → 25-85% (transcode) → 85% (thumbnail) → 100% (complete)

### Session Authentication
Tower-sessions with SQLite backend:
- Session cookies: HttpOnly, SameSite=Lax
- OIDC integration via `user-auth` crate
- Emergency login support (`ENABLE_EMERGENCY_LOGIN=true`)

### Database Schema Quirks
- **SQLite-specific features** - No PostgreSQL compatibility
- **`media_items.id` is `INTEGER PRIMARY KEY`** - SQLite auto-assigns on insert; do not bind an explicit id value, use `last_insert_rowid()` after execute
- All other tables use `INTEGER PRIMARY KEY AUTOINCREMENT`
- **Status field** - `processing` → `active` or `error` for async operations
- **video_type field** - `mp4` (direct playback) vs `hls` (streaming)

### System Dependencies
Required external tools (must be in PATH):
- `ffmpeg` - Video transcoding
- `ffprobe` - Video metadata extraction
- `mediamtx` - RTMP/HLS streaming server
- `gs` (Ghostscript) - PDF thumbnail generation
- `cwebp` - WebP image conversion

## Common Development Tasks

### Media Serving Routes (media-manager)
All serving goes through `/media/{slug}/...` — legacy `/images/`, `/videos/`, `/documents/` routes have been removed:

| Route | Handler | Notes |
|---|---|---|
| `GET /media/{slug}/image.webp` | `serve_image_webp` | Serves WebP; falls back to original (including SVG) |
| `GET /media/{slug}/thumbnail` | `serve_thumbnail` | All types; images fall back to WebP if no thumb |
| `GET /media/{slug}/video.mp4` | `serve_video_mp4` | MP4 direct playback only |
| `GET /media/{slug}/serve` | `serve_pdf_handler` | PDF inline serving |
| `GET /hls/{slug}/{*path}` | (video-manager) | HLS segments and playlists |

**`thumbnail_url` field in `media_items`:**
- Images (WebP): `/media/{slug}/image.webp` — used as primary display and thumbnail
- Images (SVG): `/media/{slug}/image.webp` — endpoint falls back to `.svg` file
- Videos (MP4/HLS): `/media/{slug}/thumbnail` — points to generated WebP thumbnail
- Documents: `NULL`

### Adding a New Media Type
1. Update `common::storage::MediaType` enum
2. Add detection logic in `media-core`
3. Update `media-manager` upload handler
4. Add serving routes in `media-manager::serve`
5. Update templates for display

### Adding a New Crate
1. Create under `crates/` with `Cargo.toml`
2. Add to workspace members in root `Cargo.toml`
3. Define public API in `lib.rs`
4. Export routes function if needed (e.g., `pub fn foo_routes() -> Router`)
5. Wire into `src/main.rs`

### Modifying HLS Transcoding
- Core logic: `crates/video-manager/src/hls.rs`
- Quality presets: `QUALITY_PRESETS` constant
- Progress reporting: Update `ProgressTracker` at each stage
- Error handling: Always capture FFmpeg stderr with `.output()`

### Adding WebSocket Endpoints
1. Enable `ws` feature in Axum (`Cargo.toml`)
2. Add route: `.route("/path", get(ws_handler))`
3. Handler: `WebSocketUpgrade -> impl IntoResponse`
4. Use `socket.send(Message::Text(data.into()))` (Axum 0.8 requires `.into()` for Utf8Bytes)

## Environment Variables

```bash
# Required
DATABASE_URL=sqlite:media.db
STORAGE_DIR=./storage

# OIDC (optional)
OIDC_ISSUER=https://auth.example.com
OIDC_CLIENT_ID=your_client_id
OIDC_CLIENT_SECRET=your_secret
OIDC_REDIRECT_URI=http://localhost:3000/auth/callback

# Emergency login (dev only)
ENABLE_EMERGENCY_LOGIN=true

# OpenTelemetry (optional)
OTLP_ENDPOINT=http://localhost:4317

# Production mode
RUN_MODE=production  # Enforces security checks
```

## Known Issues & Gotchas

### Askama 0.13 Migration
- Feature renamed: `"serde-json"` → `"serde_json"`
- `askama_axum` crate removed, use `Html(template.render()?)`
- Templates don't implement `IntoResponse`

### Tower Governor 0.8
- `GovernorLayer` has 3 generics: `<K, M, RespBody>`
- Construct via `GovernorLayer::new(Arc::new(config))`
- Use `NoOpMiddleware` from `governor` 0.10

### Migration System
- Manual migration required for numbered files
- Auto-migration only works for timestamp-versioned files
- Check `_sqlx_migrations` table to verify applied migrations

### WebSocket Message Types
Axum 0.8 uses `Utf8Bytes` for `Message::Text`:
```rust
// Correct
sender.send(Message::Text(json_string.into())).await

// Wrong
sender.send(Message::Text(json_string)).await  // Type error
```

### FFmpeg Threads
Setting `threads: 0` lets FFmpeg auto-detect optimal thread count. Don't hardcode thread counts.

## Performance Considerations

- **HLS Transcoding:** CPU-intensive, run in background with `tokio::spawn`
- **WebSocket Polling:** 500ms interval balances responsiveness vs server load
- **Rate Limiting:** Adjust per-route based on resource cost
- **Image Serving:** WebP conversion reduces bandwidth by ~30%
- **Vault Storage:** Isolates user data, enables per-vault quotas

## Testing Notes

- Integration tests require running database
- Video tests need FFmpeg in PATH
- Mock MediaMTX responses for streaming tests
- Use `tempfile` crate for filesystem tests
- Clean up test artifacts in vault directories

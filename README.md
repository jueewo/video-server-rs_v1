# AppKask

> Deliver your consulting work in a complete environment your clients own.

You deliver process models, data platforms, training materials, and media assets. Today that means seven tools, seven logins, seven places where client data lives. That's not a delivery — it's a scavenger hunt.

AppKask packages your entire consulting delivery into one self-hosted platform. One workspace per client. One link to share. One binary to hand over.

---

## What It Does

**Workspace > Folder > App.** Each client gets a workspace. Inside, folders are typed — assign *media-server* and it becomes a media gallery. Assign *course* and it becomes a training environment. Assign *bpmn-simulator* and it becomes a process modeler. You never leave the workspace. Your client never sees infrastructure.

**Share without accounts.** Generate an access code for any folder. Your client opens a URL, enters the code, and browses the deliverables. No sign-up, no IT approval, no friction.

**Hand over the whole thing.** When the engagement ends, ship the client a standalone instance running on their server. Their data, your work product, packaged as a branded platform.

---

## Features

| Capability | What you get |
|---|---|
| **Media Management** | Upload, transcode, stream. HLS adaptive bitrate (1080p-360p), real-time progress, auto-thumbnails |
| **Site Generator** | Build multi-language client websites from structured data. Astro-powered, theme-able |
| **Process Modeling** | Interactive BPMN diagrams your clients can explore and simulate |
| **Course Delivery** | Training programs with Reveal.js presentations and structured modules |
| **3D Gallery** | Present deliverables in an immersive WebGL space (Babylon.js) |
| **Custom Tools** | Deploy your Vue3/Preact data platforms as workspace folders |
| **Live Streaming** | RTMP ingest via MediaMTX, HLS output, WebRTC for low latency |
| **Access Control** | Four layers: public, access codes, groups (RBAC), ownership. Full audit trail |
| **WebDAV** | Clients drag-drop files from Finder or Explorer |

---

## Quick Start

### Docker (recommended)

```bash
git clone https://github.com/jueewo/appkask.git
cd appkask/docker
docker compose up -d
open http://localhost:3000
```

### Native

**Prerequisites:** Rust, FFmpeg, ffprobe, MediaMTX, Ghostscript, cwebp

```bash
# macOS
brew install ffmpeg mediamtx ghostscript webp

# Start MediaMTX (separate terminal)
mediamtx mediamtx.yml

# Start AppKask
cargo run --release
open http://localhost:3000
```

---

## Architecture

A Cargo workspace with 34 crates, built on Axum 0.8 and SQLite.

```
appkask
 |-- Axum HTTP server (port 3000)
 |-- MediaMTX streaming server (RTMP/HLS/WebRTC)
 |-- SQLite database (single file, WAL mode)
 |-- Vault-based storage (isolated per workspace)
```

### Key technical decisions

- **Rust** — Single binary, ~20 MB. No runtime, no garbage collector, no surprises at 2 AM.
- **SQLite** — One file to back up. Zero configuration. Handles 1-50 concurrent users with WAL mode.
- **Modular crates** — 34 workspace crates, each with a focused responsibility. Add a folder type by implementing one trait.
- **Askama templates** — Type-checked at compile time. No template errors in production.
- **OpenTelemetry** — Distributed tracing built in. Plug into Grafana, SigNoz, or Jaeger.

### Workspace crates

| Layer | Crates |
|---|---|
| **Core** | `common`, `media-core`, `access-control` |
| **Media** | `media-manager`, `video-manager`, `docs-viewer`, `bpmn-viewer`, `pdf-viewer` |
| **Auth** | `user-auth` (OIDC/Casdoor), `access-codes`, `access-groups`, `api-keys` |
| **Workspace** | `workspace-manager`, `vault-manager`, `site-generator`, `site-publisher` |
| **Apps** | `3d-gallery`, `course-viewer`, `media-mcp`, `media-cli` |
| **Infra** | `rate-limiter`, `live-streaming` |

---

## How It Works

### 1. Create a workspace for your client

Each workspace is an isolated container. Upload media, model processes, build courses, host the project website — everything lives in one place.

### 2. Assign folder types

A folder's type determines what app opens it:

| Folder type | Opens as |
|---|---|
| `media-server` | Media gallery with upload, search, tagging, streaming |
| `course` | Training environment with Reveal.js presentations |
| `bpmn-simulator` | Interactive process modeler and simulator |
| `yhm-site-data` | Multi-language static site builder (Astro) |
| `js-tool` | Your custom Vue3/Preact data platform |

### 3. Share with an access code

Generate a code. Send a link. Client enters the code and sees their deliverables. Revoke when the engagement ends.

### 4. Deliver

Three tiers, same codebase:

- **Your platform** — Multi-tenant. Each client is a scoped tenant.
- **Hosted B2B** — Client gets a branded section. Their logo, their colors. Recurring hosting revenue.
- **Standalone** — Ship the client a binary for their own server. White-label, self-contained.

---

## Configuration

### Environment variables

```bash
DATABASE_URL=sqlite:media.db        # SQLite database path
STORAGE_DIR=./storage                # File storage root

# OIDC authentication (optional)
OIDC_ISSUER=https://auth.example.com
OIDC_CLIENT_ID=your_client_id
OIDC_CLIENT_SECRET=your_secret
OIDC_REDIRECT_URI=http://localhost:3000/auth/callback

# Development
ENABLE_EMERGENCY_LOGIN=true          # Dev-only login bypass

# Observability
OTLP_ENDPOINT=http://localhost:4317  # OpenTelemetry collector

# Production
RUN_MODE=production                  # Enforces security checks
```

### Ports

| Port | Service | Purpose |
|---|---|---|
| 3000 | AppKask | Web UI, API, media serving |
| 1935 | MediaMTX | RTMP ingest |
| 8888 | MediaMTX | HLS output |
| 8889 | MediaMTX | WebRTC output |

### System dependencies

| Tool | Purpose |
|---|---|
| FFmpeg + ffprobe | Video transcoding, metadata extraction |
| MediaMTX | RTMP/HLS/WebRTC streaming |
| Ghostscript (`gs`) | PDF thumbnail generation |
| cwebp | WebP image conversion |

---

## Media Pipeline

Every file uploaded goes through a processing pipeline:

- **Video** — Upload MP4, get adaptive HLS streaming (1080p, 720p, 480p, 360p auto-selected based on source). Real-time progress via WebSocket.
- **Images** — Auto-converted to WebP. Originals preserved. SVGs served with CSP protection. Thumbnails auto-generated.
- **Documents** — PDFs viewable inline. Markdown editable in-browser. BPMN diagrams interactive.

### Storage layout

```
storage/vaults/{vault_id}/
  media/
    images/{slug}.webp
    videos/{slug}/           # HLS output (index.m3u8 + segments)
    documents/{filename}
  thumbnails/
    images/{slug}_thumb.webp
    videos/{slug}_thumb.webp
```

---

## Access Control

Four layers, designed for consulting workflows:

| Layer | Use case |
|---|---|
| **Public** | Marketing materials, portfolio pieces. Visible to everyone. |
| **Access Codes** | Per-folder sharing. Link + code. No account needed. Set expiry for time-limited engagements. |
| **Groups** | Ongoing clients get role-based access (Viewer, Contributor, Editor, Admin). |
| **Ownership** | You always have full control over your deliverables. |

Every access decision — granted or denied — is logged with user, IP, resource, and timestamp.

---

## Rate Limiting

Three tiers based on resource intensity:

| Tier | Limit | Endpoints |
|---|---|---|
| Default | 60 RPM | Most endpoints |
| Upload | 15 RPM | File uploads, transcoding |
| Serving | 300 RPM | Media delivery, thumbnails |

---

## Development

```bash
# Build
cargo build

# Build specific crate
cargo build --package media-manager

# Run tests
cargo test

# Run with tracing
OTLP_ENDPOINT=http://localhost:4317 cargo run
```

### Database migrations

SQLite migrations are applied automatically on startup via `sqlx::migrate!("./migrations")`. New migrations: add a timestamped `.sql` file to `migrations/`.

---

## Production Deployment

### Docker Compose

```bash
cd docker
docker compose up -d
```

### With Caddy (HTTPS)

```bash
caddy run  # Uses included Caddyfile
```

### Security checklist

- [ ] Configure OIDC authentication (Casdoor or compatible provider)
- [ ] Set `RUN_MODE=production`
- [ ] Enable HTTPS via reverse proxy
- [ ] Set strong RTMP publish token
- [ ] Review firewall rules (expose only port 443)
- [ ] Set up automated backups (`media.db` + `storage/`)

---

## FAQ

**What does a client need to run a standalone instance?**
A single VPS with 2-8 cores and 2-8 GB RAM. External tools in PATH: FFmpeg, ffprobe, Ghostscript, cwebp. Docker Compose is the simplest deployment.

**Can I white-label it?**
Yes. Each tenant has its own branding configuration (logo, colors, name).

**Does it support PostgreSQL?**
No. SQLite is a deliberate choice — zero configuration, single-file backup, trivial to move between servers.

**Can clients upload their own media?**
Yes. Give them a user account via OIDC and they can upload, organize, and share within their scoped workspace.

**How do I extend the platform?**
Deploy Vue3/Preact apps as *js-tool* workspace folders for quick delivery. For deeper integration, create a Rust crate implementing the `FolderTypeRenderer` trait.

---

## Built With

Rust (Axum 0.8) / SQLite (sqlx) / Askama 0.13 / MediaMTX / Astro / Babylon.js / bpmn-js / Reveal.js

---

## License

License TBD.

---

Built by a consultant who got tired of handing clients links.

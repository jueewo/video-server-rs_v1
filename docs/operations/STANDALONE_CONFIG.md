# Standalone Deployment Guide (Tier 3)

> For customers who run the platform on their own infrastructure.
> Single-tenant, full data sovereignty, no phone-home.

---

## Quick Start

```bash
# 1. Clone or unpack the deployment package
git clone ... && cd video-server-rs

# 2. Configure branding
vi branding.yaml

# 3. Configure deployment mode
vi config.yaml

# 4. Build and start (media + course features as an example)
docker compose -f docker/docker-compose.standalone.yml build \
    --build-arg FEATURES=media,course
docker compose -f docker/docker-compose.standalone.yml up -d

# 5. Check health
curl http://localhost:3000/health
```

---

## Configuration Files

### `config.yaml` — Deployment topology

```yaml
# Locks the server to a single tenant. Required for standalone deployments.
deployment_mode: standalone

# Your organisation's fixed identity (not user-configurable at runtime)
tenant_id: "acme"
tenant_name: "Acme Corp"
```

| Field | Values | Notes |
|---|---|---|
| `deployment_mode` | `standalone` / `hosted` | `standalone` = single-tenant lock |
| `tenant_id` | Any slug (no spaces) | Used internally; not shown to users |
| `tenant_name` | Display string | Shown in admin areas |
| `server_id` | UUID string | Auto-generated if omitted; identifies this server for federation |
| `server_url` | URL | Public URL; required only when federation is enabled |
| `federation_enabled` | `true` / `false` | Enable pull-based multi-server catalog sharing |
| `federation_sync_interval_minutes` | Integer | How often to pull peer catalogs (default: 15) |

### `branding.yaml` — Visual identity

```yaml
# Browser tab title, navbar, footer
name: "Acme Knowledge Hub"

# Logo shown in navbar (path relative to server root)
# Drop your file at static/custom/logo.webp and reference it here
logo: "/static/custom/logo.webp"

# Favicon (defaults to /static/favicon.svg if not set)
favicon: "/static/custom/favicon.ico"

# Primary accent color (CSS hex). Leave empty for default DaisyUI blue.
primary_color: "#0057b7"

# Support email shown in footer (leave empty to hide)
support_email: "support@acme.com"

# Meta description tag
description: "Acme internal knowledge platform"
```

---

## Feature Flags (License Scope)

The Docker image is built with only the features you have licensed:

| `FEATURES` build arg | Included modules |
|---|---|
| `full` | Everything — media pipeline, course viewer, BPMN |
| `media` | Media manager, gallery, HLS transcoding |
| `course` | Course viewer (markdown-based) |
| `bpmn` | BPMN diagram viewer |
| `media,course` | Combine as needed |

```bash
# Build a media+course image
docker compose -f docker/docker-compose.standalone.yml build \
    --build-arg FEATURES=media,course

# Build a full image (default)
docker compose -f docker/docker-compose.standalone.yml build
```

Modules not compiled in produce zero binary footprint — no routes, no UI, no code.

---

## Authentication (OIDC)

The platform supports any OpenID Connect provider. Point it at your company IdP:

```yaml
# docker-compose.standalone.yml — environment section
environment:
  - OIDC_ISSUER_URL=https://auth.acme.com
  - OIDC_CLIENT_ID=platform
  - OIDC_CLIENT_SECRET=your-secret
  - OIDC_REDIRECT_URL=https://platform.acme.com/auth/callback
```

Tested with: Casdoor, Keycloak, Auth0, Azure AD (OIDC endpoint), Okta.

### Initial Setup — Emergency Login

For the very first login before OIDC is configured:

```yaml
environment:
  - ENABLE_EMERGENCY_LOGIN=true
  - SU_USER=admin
  - SU_PWD=changeme   # change immediately after first login
```

**Disable this before going to production** — set `ENABLE_EMERGENCY_LOGIN=false` or remove the variable.

---

## Data Volumes

| Volume | Purpose | Backup priority |
|---|---|---|
| `storage/` | All media files, thumbnails, HLS segments | **Critical** |
| `media.db` | SQLite database — all metadata, access codes, users | **Critical** |
| `branding.yaml` | Visual identity config | Low (can recreate) |
| `config.yaml` | Deployment config | Low (can recreate) |
| `static/custom/` | Customer logo, favicon | Low (can recreate) |

Backup `storage/` and `media.db` together. They are consistent as long as you don't delete one independently.

---

## Reverse Proxy (Caddy — recommended)

Place Caddy in front of the platform for automatic HTTPS:

```
# Caddyfile
platform.acme.com {
    reverse_proxy localhost:3000
}
```

Or add the `caddy` service to your compose file (see commented section in `docker-compose.yml`).

---

## Streaming (MediaMTX — optional)

Only needed if the `media` feature is compiled in and you want RTMP/HLS live streaming.

The `mediamtx` service in `docker-compose.standalone.yml` is included but can be commented out if not needed.

Configure your publish token in `mediamtx.yml`:

```yaml
# mediamtx.yml
paths:
  live:
    publishQuery: token=your-secret-token
```

And set the matching env var for the app:

```yaml
environment:
  - RTMP_PUBLISH_TOKEN=your-secret-token
```

---

## Environment Variables Reference

| Variable | Default | Notes |
|---|---|---|
| `DATABASE_URL` | `sqlite:media.db` | Path to SQLite DB file |
| `STORAGE_DIR` | `./storage` | Root storage directory |
| `RUN_MODE` | `production` | Set to `development` to disable security checks |
| `RUST_LOG` | `info` | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `OIDC_ISSUER_URL` | — | OIDC provider base URL |
| `OIDC_CLIENT_ID` | — | OAuth2 client ID |
| `OIDC_CLIENT_SECRET` | — | OAuth2 client secret |
| `OIDC_REDIRECT_URL` | — | Must match redirect URI registered with IdP |
| `ENABLE_EMERGENCY_LOGIN` | `false` | Bypass OIDC for initial setup only |
| `SU_USER` | — | Emergency login username |
| `SU_PWD` | — | Emergency login password |
| `RTMP_PUBLISH_TOKEN` | — | Token for RTMP ingest (media feature only) |

---

## System Requirements

| Component | Minimum | Recommended |
|---|---|---|
| CPU | 2 vCPU | 4+ vCPU (for HLS transcoding) |
| RAM | 1 GB | 4 GB |
| Disk | 20 GB | Depends on media volume |
| OS | Linux (Docker) | Ubuntu 22.04 / Debian 12 |

Runtime dependencies (included in Docker image):
- `ffmpeg` — HLS transcoding and thumbnails (media feature)
- `ffprobe` — Video metadata extraction (media feature)
- `ghostscript` — PDF thumbnail generation (media feature)
- `cwebp` — WebP image conversion (media feature)

---

## See Also

- `docs/management/DELIVERY_TIERS.md` — full tier design
- `docs/management/ROADMAP.md` — implementation status
- `docker/docker-compose.standalone.yml` — compose file for Tier 3
- `docker/Dockerfile` — multi-stage build with `FEATURES` arg

# Docker / Podman Deployment

See **[DEPLOY.md](DEPLOY.md)** for the full step-by-step installation guide
(Rocky Linux 10, Podman, Harbor, Forgejo CI).

---

## Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Main stack — all services |
| `docker-compose.casdoor.yml` | Optional override — adds a local Casdoor OIDC instance |
| `docker-compose.standalone.yml` | Stripped-down single-app deployment (Tier 3) |
| `Dockerfile` | Multi-stage build for media-server (Rust 1.85 + bun + Alpine) |
| `Dockerfile.mcp` | Build for the media-mcp AI integration binary |
| `Caddyfile` | Reverse proxy config (central Casdoor — default) |
| `Caddyfile.casdoor` | Reverse proxy config (local Casdoor override) |
| `.env.example` | All configurable environment variables with descriptions |
| `build-push.sh` | Manual build + push to Harbor registry |
| `otel-collector-config.yml` | OpenTelemetry collector pipeline config |
| `prometheus.yml` | Prometheus scrape targets |
| `grafana/` | Grafana provisioning (datasources, dashboards) |
| `casdoor/app.conf.example` | Casdoor config template (only for local Casdoor override) |

---

## Services

| Service | Image | Ports |
|---------|-------|-------|
| `caddy` | caddy:2-alpine | 80, 443 |
| `media-server` | Harbor / local build | 3000 (internal) |
| `media-mcp` | local build | stdio only |
| `mediamtx` | bluenviron/mediamtx | 1935, 8888, 8889, 8554, 9997, 9998 |
| `otel-collector` | otel/opentelemetry-collector-contrib | 4317, 4318, 8890 (internal) |
| `prometheus` | prom/prometheus | 9090 (internal) |
| `grafana` | grafana/grafana | 3001 (internal) |
| `casdoor` _(optional)_ | casbin/casdoor | 8000 (override only) |

---

## Quick reference

```bash
# Copy and fill in config
cp .env.example .env

# Start
podman compose up -d

# With local Casdoor instead of a central one
podman compose -f docker-compose.yml -f docker-compose.casdoor.yml up -d

# Logs
podman compose logs -f media-server

# Pull a new image from Harbor and redeploy
podman compose pull media-server && podman compose up -d media-server

# Stop
podman compose down
```

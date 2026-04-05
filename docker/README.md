# Docker / Podman Deployment

See **[DEPLOY.md](DEPLOY.md)** for the full step-by-step installation guide
(Rocky Linux 10, Podman, Harbor, Forgejo CI).

---

## Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Full stack — Caddy, MediaMTX, app, MCP, OTEL, Prometheus, Grafana |
| `docker-compose.standalone.yml` | Minimal stack — app + MediaMTX (recommended for testing) |
| `docker-compose.casdoor.yml` | Optional override — adds a local Casdoor OIDC instance |
| `Dockerfile` | Multi-stage build for media-server (Rust 1.85 + bun + Alpine) |
| `Dockerfile.mcp` | Build for the media-mcp AI integration binary |
| `Caddyfile` | Reverse proxy config (central Casdoor — default) |
| `Caddyfile.casdoor` | Reverse proxy config (local Casdoor override) |
| `.env.example` | Environment variables for the full stack |
| `.env.standalone.example` | Environment variables for standalone (emergency login enabled) |
| `branding.yaml.example` | White-label identity template (name, logo, colors) |
| `config.yaml.example` | Deployment topology template (standalone mode) |
| `otel-collector-config.yml` | OpenTelemetry collector pipeline config |
| `prometheus.yml` | Prometheus scrape targets |
| `grafana/` | Grafana provisioning (datasources, dashboards) |
| `casdoor/app.conf.example` | Casdoor config template (only for local Casdoor override) |

---

## Scripts

Operational scripts live in [`scripts/`](scripts/). See
[scripts/README.md](scripts/README.md) for detailed usage.

| Script | Purpose |
|--------|---------|
| [`scripts/quickstart-test.sh`](scripts/quickstart-test.sh) | One-command test setup: creates .env, configs, empty DB, optionally starts |
| [`scripts/build-local.sh`](scripts/build-local.sh) | Build image locally, optionally export as `.tar.gz` for transfer |
| [`scripts/build-push.sh`](scripts/build-push.sh) | Build and push to a configurable Harbor registry |
| [`scripts/deploy-harbor.sh`](scripts/deploy-harbor.sh) | Build and push to `harbor.appkask.com` |
| [`scripts/verify-db.sh`](scripts/verify-db.sh) | Verify database tables and migrations after first start |

---

## Services

| Service | Image | Ports |
|---------|-------|-------|
| `caddy` | caddy:2-alpine | 80, 443 |
| `media-server` / `platform` | Harbor / local build | 3000 |
| `media-mcp` | local build | stdio only |
| `mediamtx` | bluenviron/mediamtx | 1935, 8888, 8889, 8554, 9997, 9998 |
| `otel-collector` | otel/opentelemetry-collector-contrib | 4317, 4318, 8890 |
| `prometheus` | prom/prometheus | 9090 |
| `grafana` | grafana/grafana | 3001 |
| `casdoor` _(optional)_ | casbin/casdoor | 8000 |

---

## Quick reference

```bash
# ── Test deployment (standalone, no OIDC) ──
./scripts/quickstart-test.sh --start -d
./scripts/verify-db.sh --container
# visit http://localhost:3000 — login: admin / changeme

# ── Full stack (production) ──
cp .env.example .env                # edit with your domain + OIDC credentials
podman compose up -d

# ── With local Casdoor ──
podman compose -f docker-compose.yml -f docker-compose.casdoor.yml up -d

# ── Deploy to Harbor ──
./scripts/deploy-harbor.sh 1.0.0

# ── Logs / stop ──
podman compose logs -f media-server
podman compose down
```

# Deployment Guide

Step-by-step instructions for running the media server on a Linux server using
Docker or Podman. Tested on **Rocky Linux 10**.

> Rocky Linux 10 uses **DNF5** — `yum-config-manager` is gone, use
> `dnf config-manager` instead. It also ships with **Podman 5.x** which has
> improved compose support.

---

## Quick start (test deployment)

For a quick test with emergency login (no OIDC needed):

```bash
git clone https://github.com/jueewo/video-server-rs_v1.git
cd video-server-rs_v1/docker
./scripts/quickstart-test.sh --start -d
```

This creates all config files, an empty database, and starts the standalone
stack. Visit `http://localhost:3000` and log in with `admin / changeme`.

---

## Scripts reference

| Script | Purpose |
|--------|---------|
| `scripts/quickstart-test.sh` | One-shot setup: creates `.env`, config files, empty DB, optionally builds and starts |
| `scripts/build-local.sh` | Build image locally (Mac/dev machine), optionally export as `.tar.gz` for transfer |
| `scripts/build-push.sh` | Build image and push to configurable Harbor registry |
| `scripts/deploy-harbor.sh` | Build and push to harbor.appkask.com |
| `scripts/verify-db.sh` | Check that database was initialized correctly after first start |

---

## Prerequisites

### Docker

Rocky Linux 10 is RHEL 10 compatible. Docker CE supports it via the RHEL repo:

```bash
# DNF5 — use config-manager directly (no yum-utils needed)
sudo dnf config-manager --add-repo https://download.docker.com/linux/rhel/docker-ce.repo
sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
sudo systemctl enable --now docker

# Allow your user to run docker without sudo
sudo usermod -aG docker $USER
newgrp docker
```

### Podman (recommended on Rocky Linux 10)

Podman 5.x ships in the default Rocky 10 repos — no extra repo needed:

```bash
sudo dnf install -y podman

# podman-compose is in EPEL
sudo dnf install -y epel-release
sudo dnf install -y podman-compose

# Allow rootless binding of ports 80 and 443
echo "net.ipv4.ip_unprivileged_port_start=80" | sudo tee /etc/sysctl.d/99-podman-ports.conf
sudo sysctl --system
```

Verify:

```bash
podman --version        # should show 5.x
podman-compose version
```

> **Note:** `podman-compose` ignores `depends_on: condition: service_healthy` — services
> start immediately without waiting for health checks. This is fine because all services
> have `restart: unless-stopped` and will retry automatically.

---

## Compose files

| File | What it includes |
|------|-----------------|
| `docker-compose.yml` | Full stack: Caddy (HTTPS) + MediaMTX + app + MCP + OTEL + Prometheus + Grafana |
| `docker-compose.standalone.yml` | Minimal: app + MediaMTX only (recommended for testing) |
| `docker-compose.casdoor.yml` | Override: adds local Casdoor to the full stack |

**Standalone** is the simplest way to test. It uses `.env.standalone.example` and
exposes port 3000 directly (no Caddy/HTTPS).

**Full stack** is for production with Caddy handling TLS, plus monitoring.
It uses `.env.example`.

---

## 1. Clone the repository

```bash
git clone https://github.com/jueewo/video-server-rs_v1.git
cd video-server-rs_v1
```

---

## 2. Configure DNS (full stack only)

Point two A records at your server's public IP before starting — Caddy needs them
to issue Let's Encrypt certificates automatically:

| Record | Example |
|--------|---------|
| `app.example.com` | your app |
| `auth.example.com` | Casdoor (OIDC) |

Skip this for standalone test deployments using port 3000 directly.

---

## 3. Prepare config files

All commands run from the **`docker/`** directory:

```bash
cd docker
```

### Standalone deployment

```bash
# Automated setup (creates .env, config, branding, empty DB):
./scripts/quickstart-test.sh

# Or manually:
cp .env.standalone.example .env
cp branding.yaml.example ../branding.yaml
cp config.yaml.example ../config.yaml
touch ../media.db
mkdir -p ../storage ../static/custom
```

### Full stack deployment

```bash
cp .env.example .env
```

Edit `.env` and set at minimum:

```env
DOMAIN=app.example.com
AUTH_DOMAIN=auth.example.com
GRAFANA_PASSWORD=<strong-password>
```

Create required data files and directories:

```bash
touch ../media.db
mkdir -p ../storage ../livestreams
```

---

## 4. Configure OIDC (Casdoor)

The stack uses your **existing central Casdoor instance** — no Casdoor container
is included by default.

In your central Casdoor, create an Application for this media server:

- Left menu → Applications → Add
- Name: `media-server`
- Redirect URLs: `https://app.example.com/auth/callback`
- Note the **Client ID** and **Client Secret**

To find your **User ID** (UUID needed for `PLATFORM_ADMIN_ID`):
- Left menu → Users → click your user → note the ID field

Fill in `.env`:

```env
OIDC_ISSUER_URL=https://your-central-casdoor.example.com
OIDC_CLIENT_ID=<Client ID>
OIDC_CLIENT_SECRET=<Client Secret>
PLATFORM_ADMIN_ID=<your User UUID>
```

> **No OIDC provider?** Use emergency login for testing:
> set `ENABLE_EMERGENCY_LOGIN=true`, `SU_USER=admin`, `SU_PWD=changeme` in `.env`.
> The standalone `.env.standalone.example` has this enabled by default.
>
> **No central Casdoor?** Run one locally using the optional override:
> see [Local Casdoor](#local-casdoor-optional) at the bottom of this guide.

---

## 5. Start the stack

### Standalone (test)

```bash
podman compose -f docker-compose.standalone.yml up -d --build
```

Visit `http://localhost:3000`.

### Full stack (production)

```bash
podman compose up -d
```

Caddy will automatically obtain a TLS certificate for `DOMAIN` on first start.
This requires port 80 and 443 to be reachable from the internet.

### Verify

```bash
podman compose ps                           # check all containers are up
podman compose logs -f platform             # standalone
podman compose logs -f media-server         # full stack

# Verify database was initialized
./scripts/verify-db.sh                              # check on host
./scripts/verify-db.sh --container                  # check inside container
```

The app is ready when the health check passes:

```
platform  healthy
```

---

## 6. Verify admin access

After logging in, visit `/profile` — your **User ID** should match the
`PLATFORM_ADMIN_ID` you set. If it does, the **Platform Admin** card appears
and `/admin` is accessible.

---

## Building the image

### Option A — build on your dev machine, transfer to server

Rust compilation is CPU/RAM intensive. Build on your Mac or dev machine instead
of the server:

```bash
cd docker

# Build locally
./scripts/build-local.sh

# Build + export as tarball for transfer
./scripts/build-local.sh --export

# Transfer to server
scp media-server.tar.gz user@server:~/
ssh user@server 'podman load < media-server.tar.gz'
```

### Option B — build and push to Harbor

```bash
cd docker

# Build and push :latest + :<git-sha>
./scripts/build-push.sh

# Build and push a versioned release
./scripts/build-push.sh 1.2.0
```

### Option C — Forgejo CI

The workflow at `.forgejo/workflows/docker-build.yml` triggers in two ways:

**Manual** (recommended — build when you decide it's ready):

1. Forgejo → your repo → Actions → **Build & Push to Harbor** → Run workflow
2. Optionally enter a version tag
3. The action builds, pushes to Harbor, and prints the deploy command

**On git tag**:

```bash
git tag v1.2.0
git push origin v1.2.0   # triggers the workflow automatically
```

**Required Forgejo secrets / variables** (repo or org level):

| Type | Name | Example |
|------|------|---------|
| Variable | `HARBOR_URL` | `harbor.example.com` |
| Variable | `HARBOR_PROJECT` | `media-server` |
| Secret | `HARBOR_USER` | `robot$media-server` |
| Secret | `HARBOR_PASSWORD` | robot account secret |

Set them at: Forgejo → repo → Settings → Secrets and Variables

---

## Firewall

Open the required ports:

```bash
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --permanent --add-port=1935/tcp   # RTMP (live streaming)
sudo firewall-cmd --reload
```

The following ports are internal (not exposed to the internet):

| Port | Service |
|------|---------|
| 3000 | media-server (proxied by Caddy) |
| 8888 | MediaMTX HLS |
| 8889 | MediaMTX WebRTC |
| 9090 | Prometheus |
| 3001 | Grafana |

---

## Updating

### Pull from Harbor (recommended for production)

```bash
cd docker
podman compose pull        # pull latest image from Harbor
podman compose up -d       # recreate changed containers
```

### Build from source

```bash
git pull
podman compose up -d --build
```

### Transfer from dev machine

```bash
# On dev machine
cd docker && ./scripts/build-local.sh --export

# On server
podman load < media-server.tar.gz
cd project/docker
podman compose -f docker-compose.standalone.yml up -d
```

---

## Useful commands

```bash
# Rebuild and restart after a code change
podman compose -f docker-compose.standalone.yml up -d --build app

# View logs
podman compose -f docker-compose.standalone.yml logs -f
podman compose -f docker-compose.standalone.yml logs -f app

# Stop everything
podman compose -f docker-compose.standalone.yml down

# Stop and remove volumes (destructive — deletes all data)
podman compose -f docker-compose.standalone.yml down -v

# Open a shell in the running container
podman exec -it platform sh

# Verify database
./scripts/verify-db.sh --container
```

---

## Troubleshooting

**Caddy fails to get a certificate**
- Check that DNS is pointing at your server: `dig app.example.com`
- Check that ports 80/443 are open: `sudo firewall-cmd --list-ports`
- Check Caddy logs: `podman compose logs caddy`

**media-server returns 502 via Caddy**
- Check the app is running: `podman compose ps`
- Check app logs: `podman compose logs media-server`
- Verify health: `curl http://localhost:3000/health`

**OIDC login fails / redirect mismatch**
- Confirm the redirect URL in Casdoor matches `https://<DOMAIN>/auth/callback` exactly
- Confirm `OIDC_CLIENT_ID` and `OIDC_CLIENT_SECRET` in `.env` match Casdoor

**403 on /admin**
- Visit `/profile` and copy the **User ID** field
- Set `PLATFORM_ADMIN_ID=<that UUID>` in `.env`
- Restart: `podman compose up -d media-server`

**Port 80/443 permission denied (Podman rootless)**
```bash
echo "net.ipv4.ip_unprivileged_port_start=80" | sudo tee /etc/sysctl.d/99-podman-ports.conf
sudo sysctl --system
```

**SELinux permission denied on volume mounts (Rocky/RHEL)**

The standalone compose uses `:z` on bind mounts. If using the full stack compose
and seeing permission errors, add `:z` to bind-mount volumes:
```yaml
volumes:
  - ../media.db:/app/media.db:z
```

**Database empty after first start**
```bash
./scripts/verify-db.sh --container
```
If no tables exist, check the app logs for migration errors:
```bash
podman compose logs app | grep -i migrat
```

---

## Local Casdoor (optional)

Only needed if you don't have a central Casdoor instance.

**1. Prepare Casdoor config:**

```bash
cp casdoor/app.conf.example casdoor/app.conf
# Edit casdoor/app.conf — set origin = https://auth.example.com
```

**2. Add to `.env`:**

```env
AUTH_DOMAIN=auth.example.com
OIDC_ISSUER_URL=http://casdoor:8000
```

**3. Start with the Casdoor override:**

```bash
# Start Casdoor first for initial setup
podman compose -f docker-compose.yml -f docker-compose.casdoor.yml up -d casdoor

# Visit http://<server>:8000 — log in with admin / 123456
# Change the password, create an Organisation, Application, and your user.
# Copy Client ID / Secret into .env.

# Then start the full stack
podman compose -f docker-compose.yml -f docker-compose.casdoor.yml up -d
```

The override adds the `casdoor` container and switches Caddy to `Caddyfile.casdoor`
which also proxies `AUTH_DOMAIN` → `casdoor:8000`.

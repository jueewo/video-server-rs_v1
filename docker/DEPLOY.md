# Deployment Guide

Step-by-step instructions for running the media server on a Linux server using
Docker or Podman. Tested on **Rocky Linux 10**.

> Rocky Linux 10 uses **DNF5** — `yum-config-manager` is gone, use
> `dnf config-manager` instead. It also ships with **Podman 5.x** which has
> improved compose support.

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

## 1. Clone the repository

```bash
git clone https://github.com/jueewo/video-server-rs_v1.git
cd video-server-rs_v1
```

---

## 2. Configure DNS

Point two A records at your server's public IP before starting — Caddy needs them
to issue Let's Encrypt certificates automatically:

| Record | Example |
|--------|---------|
| `app.example.com` | your app |
| `auth.example.com` | Casdoor (OIDC) |

---

## 3. Prepare config files

All commands run from the **`docker/`** directory:

```bash
cd docker
```

**Environment file:**

```bash
cp .env.example .env
```

Edit `.env` and set at minimum:

```env
DOMAIN=app.example.com
AUTH_DOMAIN=auth.example.com
GRAFANA_PASSWORD=<strong-password>
```

Leave `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, and `PLATFORM_ADMIN_ID` blank for now —
you will fill them in after Casdoor is configured (step 5).

**Casdoor config:**

```bash
cp casdoor/app.conf.example casdoor/app.conf
```

Edit `casdoor/app.conf` and set the `origin` to your public auth URL:

```ini
origin = https://auth.example.com
```

**Create required data files and directories:**

```bash
touch ../media.db
mkdir -p ../storage ../livestreams
```

---

## 4. Start Casdoor for initial setup

```bash
# Docker
docker compose up -d casdoor

# Podman
podman compose up -d casdoor
```

Wait a few seconds, then visit `http://<server-ip>:8000` in your browser.

> Caddy is not running yet, so use the direct port for this step.

---

## 5. Configure Casdoor

1. Log in with the default credentials: **admin / 123456**
2. **Change the admin password immediately** — top-right menu → Manage My Account

**Create an Organization:**

- Left menu → Organizations → Add
- Name: `my-org` (or anything you like)

**Create an Application:**

- Left menu → Applications → Add
- Name: `media-server`
- Organization: select the one you just created
- Client ID / Client Secret: note these down — you will add them to `.env`
- Redirect URLs: add `https://app.example.com/auth/callback`
- Save

**Create your user account:**

- Left menu → Users → Add
- Fill in your username and email
- Set a password
- Note the **User ID** (UUID) shown in the user detail page — you need this for `PLATFORM_ADMIN_ID`

---

## 6. Fill in the remaining .env values

```env
OIDC_CLIENT_ID=<Client ID from Casdoor>
OIDC_CLIENT_SECRET=<Client Secret from Casdoor>
PLATFORM_ADMIN_ID=<your User UUID from Casdoor>
```

---

## 7. Start the full stack

```bash
# Docker
docker compose up -d

# Podman
podman compose up -d
```

Caddy will automatically obtain TLS certificates for both domains on first start.
This requires ports 80 and 443 to be reachable from the internet.

Check that everything came up:

```bash
# Docker
docker compose ps
docker compose logs -f media-server

# Podman
podman compose ps
podman compose logs -f media-server
```

The app is ready when the media-server health check passes:

```
media-server  healthy
```

Visit `https://app.example.com` — you should be redirected to Casdoor to log in.

---

## 8. Verify admin access

After logging in, visit `/profile` — your **User ID** should match the
`PLATFORM_ADMIN_ID` you set. If it does, the **Platform Admin** card appears
and `/admin` is accessible.

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
| 8000 | Casdoor (proxied by Caddy) |
| 8888 | MediaMTX HLS |
| 8889 | MediaMTX WebRTC |
| 9090 | Prometheus |
| 3001 | Grafana |

---

## Useful commands

```bash
# Rebuild and restart after a code change
docker compose up -d --build media-server

# View logs
docker compose logs -f
docker compose logs -f media-server

# Stop everything
docker compose down

# Stop and remove volumes (destructive — deletes all data)
docker compose down -v

# Open a shell in the running container
docker compose exec media-server sh

# Apply a new database migration
docker compose exec media-server sqlite3 /app/media.db < /app/migrations/XXXX_name.sql
```

---

## Updating

### Option A — pull from Harbor (recommended for production)

```bash
cd docker
podman compose pull        # pull latest image from Harbor
podman compose up -d       # recreate changed containers
```

### Option B — build from source (local / no Harbor)

```bash
git pull
podman compose up -d --build
```

---

## Harbor image registry

Instead of building Rust on the server, build once on your dev machine (or via
Forgejo CI) and push to Harbor. The server only needs to pull and run.

### One-time Harbor setup

1. Log in to your Harbor instance
2. Create a project — e.g. `media-server`
3. Create a robot account with push access to that project
4. Note the robot username and secret

### Add Harbor vars to `.env`

```env
MEDIA_SERVER_IMAGE=harbor.example.com/media-server/app:latest
HARBOR_URL=harbor.example.com
HARBOR_PROJECT=media-server
HARBOR_USER=robot$media-server
HARBOR_PASSWORD=<robot-secret>
```

`MEDIA_SERVER_IMAGE` tells `podman compose` to pull the pre-built image instead
of building from source. The `HARBOR_*` vars are only needed for the build
machine and CI — the server only needs `MEDIA_SERVER_IMAGE`.

### Manual build & push (from your dev machine)

```bash
cd docker

# Build and push :latest + :<git-sha>
./build-push.sh

# Build and push a versioned release
./build-push.sh 1.2.0
```

### Forgejo CI — automatic builds

The workflow at `.forgejo/workflows/docker-build.yml` triggers in two ways:

**Manual** (recommended — build when you decide it's ready):

1. Forgejo → your repo → Actions → **Build & Push to Harbor** → Run workflow
2. Optionally enter a version tag and feature set
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

### Deploy after a new build

```bash
cd docker

# Pull the new image and restart only the affected container
podman compose pull media-server
podman compose up -d media-server
```

---

## Troubleshooting

**Caddy fails to get a certificate**
- Check that DNS is pointing at your server: `dig app.example.com`
- Check that ports 80/443 are open: `sudo firewall-cmd --list-ports`
- Check Caddy logs: `docker compose logs caddy`

**media-server returns 502 via Caddy**
- Check the app is running: `docker compose ps`
- Check app logs: `docker compose logs media-server`
- Verify health: `curl http://localhost:3000/health`

**OIDC login fails / redirect mismatch**
- Confirm the redirect URL in Casdoor matches `https://<DOMAIN>/auth/callback` exactly
- Confirm `OIDC_CLIENT_ID` and `OIDC_CLIENT_SECRET` in `.env` match Casdoor

**403 on /admin**
- Visit `/profile` and copy the **User ID** field
- Set `PLATFORM_ADMIN_ID=<that UUID>` in `.env`
- Restart: `docker compose up -d media-server`

**Port 80/443 permission denied (Podman rootless)**
```bash
echo "net.ipv4.ip_unprivileged_port_start=80" | sudo tee /etc/sysctl.d/99-podman-ports.conf
sudo sysctl --system
```

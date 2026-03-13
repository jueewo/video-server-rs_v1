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

```bash
git pull
docker compose up -d --build
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

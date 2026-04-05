# Docker / Podman Scripts

Operational scripts for building, deploying, and managing the media server.
All scripts auto-detect `podman` or `docker` and work from any directory.

Run from this folder:

```bash
cd docker/scripts
./quickstart-test.sh
```

Or from anywhere via relative/absolute path:

```bash
docker/scripts/deploy-harbor.sh 1.0.0
```

---

## Scripts

### quickstart-test.sh

One-command setup for a test deployment. Creates all required config files,
an empty database, and storage directories. Optionally builds and starts
the standalone stack.

```bash
./quickstart-test.sh              # set up files only
./quickstart-test.sh --start      # set up + build + start (foreground)
./quickstart-test.sh --start -d   # set up + build + start (detached)
```

What it creates (if missing):

| File | Source |
|------|--------|
| `docker/.env` | from `.env.standalone.example` (emergency login enabled) |
| `branding.yaml` | from `branding.yaml.example` |
| `config.yaml` | from `config.yaml.example` (standalone mode) |
| `media.db` | empty file (tables auto-created on first start) |
| `storage/` | directory for vaults and media |
| `static/custom/` | directory for custom logo/favicon |

After setup, visit `http://localhost:3000` and log in with `admin / changeme`.

---

### build-local.sh

Build the image locally without pushing to any registry. Useful for building
on your Mac and transferring the image to a server as a tarball.

```bash
./build-local.sh                   # build as media-server:local
./build-local.sh 1.2.3             # also tag as media-server:1.2.3
./build-local.sh --export          # build + save to docker/media-server.tar.gz
./build-local.sh 1.2.3 --export    # build tagged + export
```

Transfer to server:

```bash
scp docker/media-server.tar.gz user@server:~/
ssh user@server 'podman load < media-server.tar.gz'
```

---

### build-push.sh

Build the image and push to a **configurable** Harbor registry. Registry
settings are read from `docker/.env` or environment variables.

```bash
./build-push.sh                    # build + push :latest + :<sha>
./build-push.sh 1.2.3              # also push :1.2.3
```

Required environment (set in `docker/.env`):

| Variable | Example |
|----------|---------|
| `HARBOR_URL` | `harbor.example.com` |
| `HARBOR_PROJECT` | `media-server` |
| `HARBOR_USER` | `robot$media-server` |
| `HARBOR_PASSWORD` | robot account secret |

---

### deploy-harbor.sh

Build and push to **harbor.appkask.com** (hardcoded). This is the primary
deployment script for the Appkask infrastructure.

```bash
./deploy-harbor.sh                 # build + push :latest + :<sha>
./deploy-harbor.sh 1.2.3           # also tag + push :1.2.3
./deploy-harbor.sh --skip-build    # push existing image (no rebuild)
./deploy-harbor.sh -h              # show help
```

First-time setup:

1. Create a robot account in Harbor (`harbor.appkask.com` > `media-platform` > Robot Accounts)
2. Log in: `podman login harbor.appkask.com`
   or set `HARBOR_USER` / `HARBOR_PASSWORD` in `docker/.env`

Pushes to: `harbor.appkask.com/media-platform/app`

Tags pushed: `:latest`, `:<git-sha>`, and optionally `:<version>`.

---

### verify-db.sh

Check that the database was initialized correctly after first start.
Verifies tables exist, migrations were applied, and key tables are accessible.

```bash
./verify-db.sh                     # check media.db on host (needs sqlite3)
./verify-db.sh --container         # check inside the running 'platform' container
```

Example output:

```
==> Checking database: /path/to/media.db

[ok] Tables found:
    _sqlx_migrations  media_items  users  vaults  ...

[ok] Migrations applied: 23 (latest: add_workspace_apps)
[ok] media_items: 42 rows
[ok] users: 3 rows
[ok] vaults: 2 rows

==> Database looks good.
```

---

## Typical workflows

### Test deployment (no OIDC, no registry)

```bash
cd docker/scripts
./quickstart-test.sh --start -d
./verify-db.sh --container
# visit http://localhost:3000 — login: admin / changeme
```

### Build on Mac, deploy to server

```bash
cd docker/scripts
./build-local.sh --export
scp ../media-server.tar.gz user@server:~/
ssh user@server 'podman load < media-server.tar.gz'
ssh user@server 'cd project/docker && podman compose -f docker-compose.standalone.yml up -d'
```

### Deploy via Harbor

```bash
cd docker/scripts
./deploy-harbor.sh 1.0.0

# On server:
cd docker
podman compose pull && podman compose up -d
```

# Docker Deployment Guide

**Last Updated:** January 2026

Complete guide for running the media server in Docker with separate services:
- **media-server** - Rust application (Alpine Linux + FFmpeg)
- **mediamtx** - Streaming server (official MediaMTX image)

---

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Building the Image](#building-the-image)
- [Running with Docker Compose](#running-with-docker-compose)
- [Services](#services)
- [Configuration](#configuration)
- [Volumes and Persistence](#volumes-and-persistence)
- [Environment Variables](#environment-variables)
- [Ports](#ports)
- [Health Checks](#health-checks)
- [Troubleshooting](#troubleshooting)
- [Production Deployment](#production-deployment)

---

## 🚀 Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/yourusername/video-server-rs_v1.git
cd video-server-rs_v1

# 2. Build and start with docker-compose
docker-compose up -d

# 3. Access the application
open http://localhost:3000

# 4. View logs
docker-compose logs -f
```

That's it! Both services are now running:
- `media-server` - http://localhost:3000
- `mediamtx` - Streaming backend

---

## 🏛️ Architecture

### Two-Service Design

This deployment uses **separate containers** for better architecture:

```
┌─────────────────┐         ┌──────────────────┐
│   media-server  │ ◄─────► │    mediamtx      │
│  (Rust + FFmpeg)│         │ (Streaming Server)│
│   Port: 3000    │         │  Ports: 1935-9998│
└─────────────────┘         └──────────────────┘
        │                            │
        └────────────┬───────────────┘
                     ▼
              Docker Network
              (media-network)
```

**Benefits:**
- ✅ **Independent scaling** - Scale each service separately
- ✅ **Easy maintenance** - Update services independently
- ✅ **Clear separation** - Media server vs streaming server
- ✅ **Official MediaMTX image** - Always up-to-date
- ✅ **Resource isolation** - CPU/memory limits per service
- ✅ **Health monitoring** - Per-service health checks

### Services

| Service | Image | Purpose | Ports |
|---------|-------|---------|-------|
| `media-server` | Custom (Alpine + Rust) | Web UI, API, transcoding | 3000 |
| `mediamtx` | bluenviron/mediamtx:latest | RTMP/HLS/WebRTC streaming | 1935, 8888, 8889, 9997, 9998 |

---

## ✅ Prerequisites

- **Docker:** 20.10+ 
- **Docker Compose:** 2.0+ (optional but recommended)
- **Available ports:** 3000, 1935, 8888, 8889, 9997, 9998
- **Disk space:** ~2GB for image, more for storage

### Install Docker

**Ubuntu/Debian:**
```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
```

**macOS:**
```bash
brew install --cask docker
```

**Windows:**
Download Docker Desktop from https://www.docker.com/products/docker-desktop

---

## 🏗️ Building the Image

### Using Docker Compose (Recommended)

```bash
# Build the image
docker-compose build

# Build with no cache (clean build)
docker-compose build --no-cache
```

### Using Docker CLI

```bash
# Build the media-server image
docker build -t media-server:latest .

# Build for specific platform
docker build --platform linux/amd64 -t media-server:latest .

# Note: MediaMTX uses official image (no build needed)
```

### What Gets Built

The multi-stage build process:

1. **Builder stage (rust:alpine):**
   - Installs Node.js and npm
   - Builds Tailwind CSS
   - Compiles Rust application in release mode

2. **Runtime stage (alpine:3.19):**
   - Minimal Alpine Linux base
   - FFmpeg and multimedia libraries
   - SQLite
   - MediaMTX
   - Only the compiled binary and assets

**Final image size:** 
- media-server: ~250MB (Alpine)
- mediamtx: ~50MB (official image)

---

## 🐳 Running with Docker Compose

### Start Services

```bash
# Start in foreground (see logs)
docker-compose up

# Start in background (detached)
docker-compose up -d

# Start with rebuild
docker-compose up --build
```

### Stop Services

```bash
# Stop containers
docker-compose stop

# Stop and remove containers
docker-compose down

# Stop, remove, and delete volumes
docker-compose down -v
```

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f media-server

# Last 100 lines
docker-compose logs --tail=100
```

### Restart Services

```bash
# Restart all
docker-compose restart

# Restart specific service
docker-compose restart media-server
```

### Scale Services (Advanced)

```bash
# Run multiple instances (needs load balancer)
docker-compose up -d --scale media-server=3
```

---

## 🔧 Running with Docker CLI

### Basic Run

```bash
docker run -d \
  --name media-server \
  -p 3000:3000 \
  -p 1935:1935 \
  -p 8888:8888 \
  -p 8889:8889 \
  -v $(pwd)/storage:/app/storage \
  -v $(pwd)/media.db:/app/media.db \
  media-server:latest
```

### With All Options

```bash
docker run -d \
  --name media-server \
  --restart unless-stopped \
  -p 3000:3000 \
  -p 1935:1935 \
  -p 8888:8888 \
  -p 8889:8889 \
  -p 9997:9997 \
  -p 9998:9998 \
  -v $(pwd)/storage:/app/storage \
  -v $(pwd)/livestreams:/app/livestreams \
  -v $(pwd)/media.db:/app/media.db \
  -e RUST_LOG=info \
  -e DATABASE_URL=sqlite:media.db \
  --health-cmd="wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1" \
  --health-interval=30s \
  --health-timeout=3s \
  --health-retries=3 \
  media-server:latest
```

### Separate Service Commands

**Start media-server only:**
```bash
docker-compose up -d media-server
```

**Start MediaMTX only:**
```bash
docker-compose up -d mediamtx
```

**Shell access:**
```bash
# Media server
docker exec -it media-server /bin/sh

# MediaMTX
docker exec -it mediamtx /bin/sh
```

**Run FFmpeg in media-server:**
```bash
docker exec media-server ffmpeg -version
```

---

## ⚙️ Configuration

### Media Server Environment Variables

Set in `docker-compose.yml` under `media-server` service:

```yaml
environment:
  # Logging
  - RUST_LOG=info                    # debug, info, warn, error
  
  # Database
  - DATABASE_URL=sqlite:media.db
  
  # Server
  - PORT=3000                        # HTTP port
  
  # MediaMTX connection (internal)
  - MEDIAMTX_HOST=mediamtx
  - MEDIAMTX_API_PORT=9997
  - MEDIAMTX_HLS_PORT=8888
  
  # OIDC Authentication (optional)
  - OIDC_ISSUER_URL=https://auth.example.com
  - OIDC_CLIENT_ID=your-client-id
  - OIDC_CLIENT_SECRET=your-secret
  - OIDC_REDIRECT_URL=https://media.example.com/auth/callback
  
  # Emergency Login (disable in production!)
  - ENABLE_EMERGENCY_LOGIN=false
  - SU_USER=admin
  - SU_PWD=changeme
```

### MediaMTX Configuration

MediaMTX uses its own configuration file:

```yaml
# In docker-compose.yml
mediamtx:
  volumes:
    - ./mediamtx.yml:/mediamtx.yml:ro
  environment:
    - MTX_PROTOCOLS=tcp
    - MTX_LOGLEVEL=info
```

Customize `mediamtx.yml` before starting services.

---

## 💾 Volumes and Persistence

### Required Volumes

**Media Server:**
```yaml
volumes:
  # Media storage (REQUIRED for persistence)
  - ./storage:/app/storage
  
  # Database (REQUIRED for persistence)
  - ./media.db:/app/media.db
```

**MediaMTX:**
```yaml
volumes:
  # Configuration
  - ./mediamtx.yml:/mediamtx.yml:ro
  
  # Live stream recordings
  - ./livestreams:/recordings
```

### Volume Structure

```
storage/
├── images/              # All images (flat structure)
│   ├── {uuid}.webp      # Original images
│   └── {uuid}_thumb.webp # Thumbnails
├── videos/              # All videos (flat structure)
│   └── {slug}/          # Video folders
│       └── hls/         # HLS segments and playlists
└── temp/                # Temporary upload files

livestreams/
└── live/                # Recorded live streams
    └── YYYY-MM-DD_HH-MM-SS.mp4

media.db                 # SQLite database
```

### Backup Volumes

```bash
# Backup storage
docker run --rm \
  -v media-server_storage:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/storage-$(date +%Y%m%d).tar.gz /data

# Backup database
cp media.db media.db.backup-$(date +%Y%m%d)
```

### Restore Volumes

```bash
# Restore storage
docker run --rm \
  -v media-server_storage:/data \
  -v $(pwd)/backups:/backup \
  alpine tar xzf /backup/storage-20260115.tar.gz -C /

# Restore database
cp media.db.backup-20260115 media.db
```

---

## 🔌 Ports

| Port | Container | Protocol | Description |
|------|-----------|----------|-------------|
| 3000 | media-server | HTTP | Web UI and API |
| 1935 | mediamtx | RTMP | Live stream ingest |
| 8888 | mediamtx | HTTP | HLS output |
| 8889 | mediamtx | HTTP/WebRTC | WebRTC streaming |
| 8554 | mediamtx | RTSP | RTSP streaming |
| 9997 | mediamtx | HTTP | Control API |
| 9998 | mediamtx | HTTP | Prometheus metrics |

### Port Mapping Examples

**Change HTTP port to 8080:**
```bash
docker run -d -p 8080:3000 media-server:latest
```

**Use different external RTMP port:**
```bash
docker run -d -p 1936:1935 media-server:latest
```

**Bind to specific IP:**
```bash
docker run -d -p 192.168.1.100:3000:3000 media-server:latest
```

---

## 🏥 Health Checks

### Built-in Health Check

The Dockerfile includes a health check:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1
```

### Check Container Health

```bash
# View health status
docker inspect --format='{{.State.Health.Status}}' media-server

# View health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' media-server
```

### Manual Health Check

```bash
# From host
curl http://localhost:3000/health

# From inside container
docker exec media-server wget -qO- http://localhost:3000/health
```

---

## 🔧 Troubleshooting

### Container Won't Start

**Check logs:**
```bash
docker-compose logs media-server
# or
docker logs media-server
```

**Common issues:**
- Port already in use: Change port mapping
- Volume permission issues: Check ownership
- Database locked: Stop all instances first

### CSS Not Loading

**This shouldn't happen with Docker** (CSS is built into image), but if it does:

```bash
# Rebuild image
docker-compose build --no-cache
docker-compose up -d
```

### MediaMTX Not Starting

**Check MediaMTX logs:**
```bash
docker exec media-server /usr/local/bin/mediamtx --version
```

**Test MediaMTX API:**
```bash
curl http://localhost:9997/v3/config/get
```

### FFmpeg Not Working

**Verify FFmpeg:**
```bash
docker exec media-server ffmpeg -version
```

**Test video processing:**
```bash
docker exec media-server ffmpeg -i /app/storage/videos/public/test.mp4 -t 1 -f null -
```

### Permission Issues

**Fix storage permissions:**
```bash
# On host
sudo chown -R 1000:1000 storage/ livestreams/
chmod -R 755 storage/ livestreams/
```

**Check container user:**
```bash
docker exec media-server id
# Should show: uid=1000(mediaserver) gid=1000(mediaserver)
```

### Database Issues

**Check database:**
```bash
docker exec media-server sqlite3 /app/media.db "SELECT COUNT(*) FROM videos;"
```

**Backup and reset:**
```bash
cp media.db media.db.backup
rm media.db
docker-compose restart
```

### Shell Access for Debugging

```bash
# Get shell in running container
docker exec -it media-server /bin/sh

# Start container with shell
docker run -it --rm media-server:latest shell

# Run as root (for debugging)
docker exec -it -u root media-server /bin/sh
```

---

## 🚀 Production Deployment

### Production docker-compose.yml

```yaml
version: '3.8'

services:
  media-server:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: media-server-prod
    restart: always
    ports:
      - "127.0.0.1:3000:3000"  # Only expose to localhost
    volumes:
      - /data/media-server/storage:/app/storage
      - /data/media-server/livestreams:/app/livestreams
      - /data/media-server/media.db:/app/media.db
    environment:
      - RUST_LOG=warn
      - DATABASE_URL=sqlite:media.db
      - ENABLE_EMERGENCY_LOGIN=false
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3000/health"]
      interval: 30s
      timeout: 3s
      start-period: 10s
      retries: 3
    networks:
      - backend

  caddy:
    image: caddy:2-alpine
    container_name: caddy-prod
    restart: always
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - media-server
    networks:
      - backend

networks:
  backend:
    driver: bridge

volumes:
  caddy_data:
  caddy_config:
```

### Security Best Practices

1. **Don't expose ports directly to internet:**
   ```yaml
   ports:
     - "127.0.0.1:3000:3000"  # Only localhost
   ```

2. **Use secrets for sensitive data:**
   ```yaml
   secrets:
     - oidc_secret
   ```

3. **Disable emergency login:**
   ```yaml
   environment:
     - ENABLE_EMERGENCY_LOGIN=false
   ```

4. **Use read-only volumes where possible:**
   ```yaml
   volumes:
     - ./mediamtx.yml:/app/mediamtx.yml:ro
   ```

5. **Limit resources:**
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '2'
         memory: 4G
   ```

### Monitoring

**View container stats:**
```bash
docker stats media-server
```

**Prometheus metrics (MediaMTX):**
```bash
curl http://localhost:9998/metrics
```

**Application logs:**
```bash
docker-compose logs -f --tail=100 media-server
```

### Automatic Updates

**Watchtower** (automatic container updates):
```yaml
watchtower:
  image: containrrr/watchtower
  volumes:
    - /var/run/docker.sock:/var/run/docker.sock
  command: --interval 300
```

### Load Balancing

For multiple instances, use nginx or traefik:

```yaml
nginx:
  image: nginx:alpine
  ports:
    - "80:80"
  volumes:
    - ./nginx.conf:/etc/nginx/nginx.conf:ro
  depends_on:
    - media-server-1
    - media-server-2
```

---

## 📊 Image Details

### Media Server Image
- **Base:** Alpine Linux 3.19
- **Size:** ~250MB
- **Includes:** FFmpeg, SQLite, CA certificates, timezone data
- **User:** Non-root (uid=1000)

### MediaMTX Image
- **Base:** Official bluenviron/mediamtx:latest
- **Size:** ~50MB
- **Always up-to-date:** Uses official releases
- **Includes:** RTMP, HLS, WebRTC, RTSP support

### Why Separate Images?

- ✅ Use official MediaMTX image (best practices)
- ✅ Each service has minimal dependencies
- ✅ Smaller total size
- ✅ Easier to update individually
- ✅ Better security isolation

---

## 🔄 Update Procedure

### Update Application

```bash
# 1. Pull latest code
git pull origin main

# 2. Rebuild and restart
docker-compose build
docker-compose up -d

# 3. Verify
docker-compose ps
docker-compose logs -f
```

### Update Base Image

```bash
# Pull latest base images
docker-compose pull

# Rebuild
docker-compose build --pull

# Restart
docker-compose up -d
```

---

## 📚 Additional Resources

- **Docker Documentation:** https://docs.docker.com
- **Docker Compose Reference:** https://docs.docker.com/compose/compose-file/
- **MediaMTX Documentation:** https://github.com/bluenviron/mediamtx
- **Alpine Linux:** https://alpinelinux.org

---

## 🆘 Support

For Docker-specific issues:

1. Check logs: `docker-compose logs -f`
2. Verify build: `docker-compose build --no-cache`
3. Test health: `curl http://localhost:3000/health`
4. Shell access: `docker exec -it media-server /bin/sh`

For application issues, see `TROUBLESHOOTING.md` and `DEPLOYMENT.md`.

---

**Architecture:** Two-service deployment (media-server + mediamtx)  
**Built with:** Alpine Linux + Rust 🦀 + MediaMTX 📡 + FFmpeg 🎬  
**Total Size:** ~300MB (250MB + 50MB)  
**Multi-stage Build:** ✅  
**Security:** Non-root users, isolated networks, minimal attack surface  
**Production Ready:** ✅  
**Scalable:** Independent service scaling
# Docker Deployment

This directory contains all Docker-related files for deploying the media server.

---

## 📁 Files

- **`Dockerfile`** - Multi-stage build for media-server (Alpine Linux + Rust + FFmpeg)
- **`docker-compose.yml`** - Two-service orchestration (media-server + mediamtx)
- **`.dockerignore`** - Exclude unnecessary files from Docker build
- **`DOCKER.md`** - Comprehensive Docker documentation (700+ lines)

---

## 🚀 Quick Start

From this directory:

```bash
# Start both services (media-server + mediamtx)
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

Or from project root:

```bash
cd docker
docker-compose up -d
```

---

## 🏗️ Architecture

### Two-Service Design

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

### Services

| Service | Image | Purpose | Ports |
|---------|-------|---------|-------|
| **media-server** | Custom (Alpine + Rust) | Web UI, API, transcoding | 3000 |
| **mediamtx** | bluenviron/mediamtx:latest | RTMP/HLS/WebRTC streaming | 1935, 8888, 8889, 9997, 9998 |

---

## 📦 Building

```bash
# Build media-server image
docker-compose build

# Build with no cache
docker-compose build --no-cache

# Build only media-server (mediamtx uses official image)
docker-compose build media-server
```

---

## 🎮 Running

### Start Services

```bash
# All services
docker-compose up -d

# Specific service
docker-compose up -d media-server
docker-compose up -d mediamtx

# View logs
docker-compose logs -f
docker-compose logs -f media-server
```

### Stop Services

```bash
# Stop all
docker-compose stop

# Stop and remove
docker-compose down

# Stop, remove, and delete volumes
docker-compose down -v
```

### Restart Services

```bash
# Restart all
docker-compose restart

# Restart specific service
docker-compose restart media-server
```

---

## 🔧 Configuration

### Environment Variables

Edit `docker-compose.yml` to configure:

**Media Server:**
```yaml
environment:
  - RUST_LOG=info                    # Logging level
  - DATABASE_URL=sqlite:media.db
  - MEDIAMTX_HOST=mediamtx          # Internal hostname
  - MEDIAMTX_API_PORT=9997
  - MEDIAMTX_HLS_PORT=8888
```

**MediaMTX:**
```yaml
environment:
  - MTX_PROTOCOLS=tcp
  - MTX_LOGLEVEL=info
```

### Volumes

**Required volumes** (persisted data):
```yaml
volumes:
  - ../storage:/app/storage          # Media files
  - ../media.db:/app/media.db        # Database
  - ../livestreams:/recordings       # Live stream recordings
  - ../mediamtx.yml:/mediamtx.yml:ro # MediaMTX config
```

---

## 🔌 Ports

| Port | Container | Service | Description |
|------|-----------|---------|-------------|
| 3000 | media-server | HTTP | Web UI and API |
| 1935 | mediamtx | RTMP | Live stream ingest |
| 8888 | mediamtx | HTTP | HLS output |
| 8889 | mediamtx | HTTP/WebRTC | WebRTC streaming |
| 8554 | mediamtx | RTSP | RTSP streaming |
| 9997 | mediamtx | HTTP | Control API |
| 9998 | mediamtx | HTTP | Prometheus metrics |

---

## 🏥 Health Checks

Both services include health checks:

```bash
# Check status
docker-compose ps

# View health
docker inspect --format='{{.State.Health.Status}}' media-server
docker inspect --format='{{.State.Health.Status}}' mediamtx

# Manual health check
curl http://localhost:3000/health              # media-server
curl http://localhost:9997/v3/config/get       # mediamtx
```

---

## 🐛 Troubleshooting

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f media-server
docker-compose logs -f mediamtx

# Last 100 lines
docker-compose logs --tail=100
```

### Shell Access

```bash
# Media server
docker-compose exec media-server /bin/sh

# MediaMTX
docker-compose exec mediamtx /bin/sh

# As root
docker-compose exec -u root media-server /bin/sh
```

### Rebuild Services

```bash
# Rebuild and restart
docker-compose up -d --build

# Force recreate
docker-compose up -d --force-recreate
```

### Common Issues

**Port conflicts:**
```bash
# Check what's using ports
lsof -i :3000
lsof -i :1935

# Change ports in docker-compose.yml
ports:
  - "8080:3000"  # Use port 8080 instead
```

**Permission issues:**
```bash
# Fix storage permissions on host
sudo chown -R 1000:1000 ../storage ../media.db
chmod -R 755 ../storage
```

**Container won't start:**
```bash
# Check logs
docker-compose logs media-server

# Inspect container
docker inspect media-server

# Remove and recreate
docker-compose down
docker-compose up -d
```

---

## 🚀 Production Deployment

### Security Recommendations

1. **Don't expose ports directly:**
   ```yaml
   ports:
     - "127.0.0.1:3000:3000"  # Only localhost
   ```

2. **Use secrets for sensitive data:**
   ```yaml
   secrets:
     - oidc_client_secret
   ```

3. **Enable HTTPS with Caddy:**
   - Uncomment Caddy service in docker-compose.yml
   - Configure domain in Caddyfile

4. **Set resource limits:**
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '2'
         memory: 4G
   ```

5. **Use production logging:**
   ```yaml
   environment:
     - RUST_LOG=warn  # Not debug or trace
   ```

### Monitoring

```bash
# View resource usage
docker stats

# View metrics (MediaMTX)
curl http://localhost:9998/metrics

# View API status
curl http://localhost:9997/v3/paths/list
```

---

## 🔄 Updates

### Update Application

```bash
# From project root
cd docker
git pull origin main
docker-compose build media-server
docker-compose up -d
```

### Update MediaMTX

```bash
# Pull latest official image
docker-compose pull mediamtx
docker-compose up -d mediamtx
```

### Update All

```bash
git pull origin main
docker-compose build
docker-compose pull
docker-compose up -d
```

---

## 📚 Documentation

- **`DOCKER.md`** - Complete Docker guide (in this directory)
- **`../DEPLOYMENT.md`** - General deployment guide
- **`../README.md`** - Main project README

---

## 🆘 Support

1. Check logs: `docker-compose logs -f`
2. Verify health: `docker-compose ps`
3. See full documentation: `DOCKER.md`
4. Test connectivity: `curl http://localhost:3000/health`

---

## 📊 Benefits

✅ **Isolated Services** - Independent scaling and updates  
✅ **Official MediaMTX** - Always up-to-date streaming server  
✅ **Small Images** - ~300MB total (Alpine-based)  
✅ **Health Checks** - Automatic restart on failure  
✅ **Easy Deployment** - One command to start everything  
✅ **Production Ready** - Non-root users, secure defaults  

---

**Quick Start:** `docker-compose up -d`  
**View Logs:** `docker-compose logs -f`  
**Access App:** http://localhost:3000  
**Documentation:** See `DOCKER.md` for complete guide
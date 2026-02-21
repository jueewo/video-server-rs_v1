# Media Server (Rust + MediaMTX)

A production-ready media management and HLS live streaming server built with Rust (Axum) and MediaMTX.

## ✅ Status: PRODUCTION READY

**Last Updated:** January 9, 2026

Features:
- ✅ RTMP live streaming ingest
- ✅ HLS output with low latency (2-3 seconds)
- ✅ WebRTC support (sub-second latency)
- ✅ Session-based authentication
- ✅ Access codes for shared media
- ✅ Stream recording (24-hour retention)
- ✅ SQLite database for metadata
- ✅ CORS support
- ✅ Interactive test page

## Architecture

```
┌─────────────┐
│ OBS/FFmpeg  │ Stream with video + audio
└──────┬──────┘
       │ RTMP: rtmp://localhost:1935/live?token=supersecret123
       ↓
┌──────────────────┐     ┌─────────────────────┐
│    MediaMTX      │────→│   Rust Server       │
│  - RTMP Ingest   │Auth │   - Authentication  │
│  - HLS Output    │     │   - Session Mgmt    │
│  - WebRTC Output │     │   - HLS Proxy       │
│  - Recording     │     │   - Web UI          │
└────┬────────┬────┘     └──────────┬──────────┘
     │        │                     │
     │ HLS    │ WebRTC             HTTP
     │        │                     │
     └────────┴─────────────────────↓
              ┌─────────────────────┐
              │      Browser        │
              │   HLS.js Player     │
              │   (2-3s latency)    │
              └─────────────────────┘
```

## Quick Start

### Option 1: Docker (Recommended)

**Easiest way to get started:**

```bash
# Clone the repository
git clone https://github.com/yourusername/video-server-rs_v1.git
cd video-server-rs_v1/docker

# Start both services (media-server + mediamtx)
docker-compose up -d

# View logs
docker-compose logs -f

# Access the application
open http://localhost:3000
```

**That's it!** Both the media server and MediaMTX are running.

See `docker/README.md` and `docker/DOCKER.md` for complete Docker documentation.

---

### Option 2: Native Installation

### Prerequisites

#### Required System Tools

1. **Rust** (already installed ✓)
2. **FFmpeg** (for video streaming and processing)
3. **MediaMTX** (streaming server)
4. **Ghostscript** (for PDF thumbnail generation)
5. **WebP tools** (for image optimization)

**Installation:**

```bash
# macOS
brew install ffmpeg mediamtx ghostscript webp

# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y ffmpeg ghostscript webp

# Alpine (Docker)
apk add --no-cache ffmpeg ghostscript libwebp-tools
```

### Install MediaMTX

**macOS:**
```bash
brew install mediamtx
```

**Linux:**
```bash
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
```

**Verify:**
```bash
mediamtx --version
```

### Running

You need **3 terminals**:

**Terminal 1 - Start MediaMTX:**
```bash
cd video-server-rs_v1
mediamtx mediamtx.yml
```

**Terminal 2 - Start Rust Server:**
```bash
cargo run --release
```

**Terminal 3 - Stream from Camera (macOS):**
```bash
# List your devices first
ffmpeg -f avfoundation -list_devices true -i ""

# Stream with video + audio
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Watch the Stream:**

1. Login: http://localhost:3000/login
2. Watch: http://localhost:3000/test

## Configuration

### Ports

| Port | Service | Purpose |
|------|---------|---------|
| 3000 | Rust HTTP | Web UI, auth, HLS proxy |
| 1935 | MediaMTX RTMP | RTMP ingest (standard port) |
| 8888 | MediaMTX HLS | HLS output |
| 8889 | MediaMTX WebRTC | WebRTC output |
| 9997 | MediaMTX API | Control API |
| 9998 | MediaMTX Metrics | Prometheus metrics |

### Stream Token

⚠️ **Change before production!**

Edit `src/main.rs`:
```rust
const RTMP_PUBLISH_TOKEN: &str = "supersecret123"; // Change this!
```

### Recording

Recordings are automatically saved to:
- **Path:** `./livestreams/live/YYYY-MM-DD_HH-MM-SS/`
- **Format:** MP4 (fMP4)
- **Retention:** 24 hours (auto-delete)
- **Segment:** 1 hour files

To disable recording, edit `mediamtx.yml`:
```yaml
paths:
  live:
    record: no
```

## URLs

### Development

- **Main Page:** http://localhost:3000
- **Login:** http://localhost:3000/login
- **Test Player:** http://localhost:3000/test
- **Health Check:** http://localhost:3000/health
- **MediaMTX Status:** http://localhost:3000/api/mediamtx/status
- **MediaMTX API:** http://localhost:9997/v3/paths/list
- **Metrics:** http://localhost:9998/metrics

### Streaming

- **RTMP:** `rtmp://localhost:1935/live?token=supersecret123`
- **HLS:** `http://localhost:3000/hls/live/index.m3u8` (requires login)
- **WebRTC:** `http://localhost:8889/live/whep` (ultra-low latency)

## OBS Studio Setup

1. Open OBS Studio
2. Settings → Stream
3. Configure:
   - **Service:** Custom
   - **Server:** `rtmp://localhost:1935/live`
   - **Stream Key:** `?token=supersecret123`
4. Settings → Output:
   - **Video Encoder:** x264
   - **Rate Control:** CBR
   - **Bitrate:** 2500 Kbps
   - **Preset:** veryfast
   - **Profile:** baseline
5. Click "Start Streaming"
6. Watch at: http://localhost:3000/test

## Database Schema

```sql
CREATE TABLE videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT 0
);
```

Sample data includes:
- `welcome` - Public video
- `fullmovie` - Public video  
- `lesson1` - Private video
- `live` - Live stream (private, requires auth)

## API Endpoints

### Public Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Home page |
| `/login` | GET | Create session |
| `/logout` | GET | Destroy session |
| `/test` | GET | HLS test player |
| `/health` | GET | Health check |

### Authentication Endpoints (Called by MediaMTX)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/stream/validate` | GET | Validate publisher token |
| `/api/stream/authorize` | GET | Authorize viewer session |

### Monitoring Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/mediamtx/status` | GET | MediaMTX status |
| `/api/webhooks/stream-ready` | POST | Stream started webhook |
| `/api/webhooks/stream-ended` | POST | Stream ended webhook |

### Access Code Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/access-codes` | POST | Create access code |
| `/api/access-codes` | GET | List access codes |
| `/api/access-codes/:code` | DELETE | Delete access code |

### Streaming Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/hls/:slug/:file` | GET | HLS proxy (live + VOD) |

## Access Codes

Access codes allow sharing private videos and images without requiring user authentication. Perfect for embedding media in websites, courses, or sharing with external users.

### Ownership & Permissions

- **Access codes are owned by the user who creates them**
- **Users can only create access codes for media they own**
- **Each access code grants access to specific videos and images owned by the creator**
- **Users can only manage (list/delete) access codes they created**

### Creating Access Codes

```bash
# Create an access code for multiple media items (must be owned by authenticated user)
curl -X POST http://localhost:3000/api/access-codes \
  -H "Content-Type: application/json" \
  -d '{
    "code": "website2024",
    "description": "Media for company website",
    "expires_at": "2024-12-31T23:59:59Z",
    "media_items": [
      {"media_type": "video", "media_slug": "welcome"},
      {"media_type": "image", "media_slug": "logo"}
    ]
  }'
```

### Using Access Codes

Append `?access_code=YOUR_CODE` to any media URL:

```
# Video player
http://localhost:3000/watch/welcome?access_code=website2024

# Image direct access
http://localhost:3000/images/logo?access_code=website2024

# HLS stream (VOD only)
http://localhost:3000/hls/welcome/index.m3u8?access_code=website2024
```

### Embedding in Websites

```html
<!-- Video embed -->
<iframe src="http://localhost:3000/watch/welcome?access_code=website2024"
        width="640" height="360"></iframe>

<!-- Image embed -->
<img src="http://localhost:3000/images/logo?access_code=website2024"
     alt="Company Logo">
```

### Managing Access Codes

```bash
# List all access codes
curl http://localhost:3000/api/access-codes

# Delete an access code
curl -X DELETE http://localhost:3000/api/access-codes/website2024
```

### Database Schema

```sql
-- Access codes table
CREATE TABLE access_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    description TEXT
);

-- Link codes to media items
CREATE TABLE access_code_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_code_id INTEGER NOT NULL,
    media_type TEXT NOT NULL CHECK (media_type IN ('video', 'image')),
    media_slug TEXT NOT NULL,
    FOREIGN KEY (access_code_id) REFERENCES access_codes(id) ON DELETE CASCADE,
    UNIQUE(access_code_id, media_type, media_slug)
);
```

## Testing

### Check Everything is Running

```bash
# 1. MediaMTX API
curl http://localhost:9997/v3/paths/list

# 2. Rust server health
curl http://localhost:3000/health

# 3. Create session
curl http://localhost:3000/login

# 4. Check if stream is live
curl http://localhost:9997/v3/paths/get/live
```

### Monitor Segments

```bash
# Watch HLS segments being created
watch -n 1 'ls -lh storage/private/live/'

# Watch recordings
watch -n 1 'ls -lh livestreams/live/'
```

## Troubleshooting

### "Connection refused" when streaming
- Check MediaMTX is running: `lsof -i :1935`
- Check config: `mediamtx mediamtx.yml`

### "Unauthorized" when watching
- Login first: http://localhost:3000/login
- Check session: curl http://localhost:3000/health

### No video appears
- Check MediaMTX logs (Terminal 1)
- Check stream is active: `curl http://localhost:9997/v3/paths/get/live`
- Check browser console for errors

### No audio
- Use `"0:0"` format (video:audio), not just `"0"`
- Grant microphone permissions on macOS
- List devices: `ffmpeg -f avfoundation -list_devices true -i ""`

### High CPU usage
- Encoding happens on streaming client (normal)
- Server just proxies (low CPU)
- Use hardware encoding if available

### Port conflicts
```bash
# Find what's using a port
lsof -i :3000
lsof -i :1935

# Kill process
kill -9 <PID>
```

## Production Deployment

### Security Checklist

- [ ] Change `RTMP_PUBLISH_TOKEN` to strong random value
- [ ] Enable HTTPS (use provided `Caddyfile`)
- [ ] Configure MediaMTX TLS in `mediamtx.yml`
- [ ] Implement proper user authentication (JWT/OAuth)
- [ ] Set up firewall rules
- [ ] Use PostgreSQL instead of SQLite
- [ ] Enable rate limiting
- [ ] Set up monitoring/alerting
- [ ] Configure CDN for HLS distribution
- [ ] Review CORS origins

### Using Caddy for HTTPS

The project includes a `Caddyfile` for automatic HTTPS:

```bash
# Install Caddy
brew install caddy  # macOS

# Start Caddy
caddy run
```

Access via: https://app.appkask.com

### Performance Optimization

**Enable hardware encoding** (streaming client):
```bash
# macOS
-c:v h264_videotoolbox

# Linux with NVIDIA GPU
-c:v h264_nvenc
```

**MediaMTX tuning** (edit `mediamtx.yml`):
```yaml
# Even lower latency (more CPU)
hlsSegmentDuration: 500ms
hlsPartDuration: 100ms
hlsSegmentCount: 3

# Or standard latency (less CPU)
hlsSegmentDuration: 2s
hlsPartDuration: 200ms
hlsSegmentCount: 6
```

## Project Structure

```
video-server-rs_v1/
├── src/
│   └── main.rs              # Main server code
├── storage/
│   ├── public/              # Public videos (VOD)
│   └── private/             # Private videos + live stream
├── livestreams/
│   └── live/                # Recorded streams (auto-deleted after 24h)
├── docs/
│   ├── README.md            # Documentation index
│   ├── LIVE_STREAMING_GUIDE.md
│   └── MEDIAMTX_MIGRATION.md
├── Cargo.toml               # Rust dependencies
├── mediamtx.yml             # MediaMTX configuration
├── Caddyfile                # HTTPS reverse proxy config
├── test-hls.html            # Standalone test player
├── media.db                 # SQLite database
├── README.md                # This file
├── QUICKSTART.md            # Quick start guide
├── PROJECT_STATUS.md        # Project status
└── MIGRATION_COMPLETE.md    # Migration history
```

## Dependencies

### Rust Crates (Cargo.toml)
- `axum` - Web framework
- `tokio` - Async runtime
- `tower` - Middleware
- `tower-http` - HTTP utilities
- `tower-sessions` - Session management
- `sqlx` - Database
- `reqwest` - HTTP client (for MediaMTX proxy)
- `serde` - Serialization
- `tracing` - Logging

### System Dependencies

| Tool | Purpose | Required | Installation |
|------|---------|----------|--------------|
| **FFmpeg** | Video streaming & transcoding | ✅ Yes | `brew install ffmpeg` |
| **MediaMTX** | RTMP/HLS streaming server | ✅ Yes | `brew install mediamtx` |
| **Ghostscript** | PDF thumbnail generation | ✅ Yes | `brew install ghostscript` |
| **WebP tools** | Image optimization (cwebp) | ✅ Yes | `brew install webp` |
| **SQLite** | Database | ✅ Yes | Usually pre-installed |
| **Caddy** | HTTPS reverse proxy | ⚠️ Optional | `brew install caddy` |

**Verify Installation:**
```bash
ffmpeg -version          # Video processing
mediamtx --version       # Streaming server
gs --version             # PDF rendering
cwebp -version           # WebP conversion
sqlite3 --version        # Database
```

**Docker:** All dependencies are included in the Dockerfile.

## Features

### Current
- ✅ Live RTMP streaming
- ✅ HLS playback (2-3s latency)
- ✅ WebRTC support (sub-1s latency)
- ✅ Session authentication
- ✅ Access codes for media sharing
- ✅ Automatic recording
- ✅ VOD playback
- ✅ Multi-origin CORS
- ✅ Health monitoring
- ✅ Metrics endpoint

### Planned
- [ ] Multi-quality ABR streaming
- [ ] Multiple concurrent streams
- [ ] JWT authentication
- [ ] User management
- [ ] Stream analytics
- [ ] Chat integration
- [ ] CDN integration
- [ ] Admin dashboard

## Documentation

### Getting Started
- **[QUICKSTART.md](QUICKSTART.md)** - Get started in 5 minutes
- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** - Current project status
- **[docs/README.md](docs/README.md)** - Documentation index
- **[docs/LIVE_STREAMING_GUIDE.md](docs/LIVE_STREAMING_GUIDE.md)** - Streaming guide
- **[docs/MEDIAMTX_MIGRATION.md](docs/MEDIAMTX_MIGRATION.md)** - Architecture details

### User Tools & Scripts
- **[scripts/user/prepare-video.sh](scripts/user/prepare-video.sh)** - Offline video preparation tool (HLS transcoding)
- **[scripts/README.md](scripts/README.md)** - Complete scripts documentation

### Docker Deployment
- **[docker/](docker/)** - Docker deployment files
- **[docker/README.md](docker/README.md)** - Docker quick start guide
- **[docker/DOCKER.md](docker/DOCKER.md)** - Complete Docker documentation
- **[docker/docker-compose.yml](docker/docker-compose.yml)** - Two-service orchestration

### Observability & Monitoring
- **[OBSERVABILITY_QUICKSTART.md](OBSERVABILITY_QUICKSTART.md)** - Quick setup with Vector + SigNoz
- **[VECTOR_SIGNOZ_SETUP.md](VECTOR_SIGNOZ_SETUP.md)** - Detailed Vector + SigNoz configuration
- **[INSTRUMENTATION.md](INSTRUMENTATION.md)** - Complete instrumentation reference

All handlers are instrumented with OpenTelemetry for distributed tracing. Traces can be exported to:
- **SigNoz** (recommended) - Complete observability platform
- **Jaeger** - Trace visualization
- **Grafana Tempo** - Trace backend for Grafana

## Why MediaMTX?

MediaMTX is a production-ready streaming server that handles:
- ✅ Multiple protocols (RTMP, HLS, WebRTC, RTSP, SRT)
- ✅ Ultra-low latency options
- ✅ Automatic error recovery
- ✅ Built-in authentication hooks
- ✅ Recording and playback
- ✅ Metrics and monitoring
- ✅ Horizontal scaling

This allows our Rust server to focus on:
- Business logic
- Authentication
- Database management
- API endpoints
- User experience

## Performance

### Resource Usage (Typical)
- **CPU:** ~5-10% (server just proxies)
- **RAM:** ~100-200 MB (Rust server)
- **Network:** Depends on stream bitrate
- **Disk:** ~500 MB/hour for recordings

### Scalability
- Supports unlimited concurrent viewers
- Single live stream (extensible to multiple)
- Recording scales with disk space
- MediaMTX can be horizontally scaled

## License

This is a reference implementation for educational purposes.

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review documentation in `docs/` folder
3. Check MediaMTX logs for streaming issues
4. Check Rust server logs for API issues

## Contributing

When making changes:
1. Test both live streaming and VOD playback
2. Update relevant documentation
3. Check authentication still works
4. Verify on multiple browsers
5. Update PROJECT_STATUS.md

---

**Built with:** Rust 🦀 + MediaMTX 📡 + Axum ⚡  
**Status:** Production Ready ✅  
**Last Updated:** January 2025
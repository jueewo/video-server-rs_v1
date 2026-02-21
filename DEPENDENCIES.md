# System Dependencies

This document lists all system-level dependencies required to run the video server.

## Overview

The server requires several external tools for media processing:

| Tool | Version | Purpose | Size | Required |
|------|---------|---------|------|----------|
| **FFmpeg** | 6.0+ | Video streaming & transcoding | ~100MB | ✅ Yes |
| **MediaMTX** | 1.5+ | RTMP/HLS streaming server | ~20MB | ✅ Yes |
| **Ghostscript** | 10.0+ | PDF → PNG rendering | ~50MB | ✅ Yes |
| **WebP tools** | 1.3+ | PNG → WebP conversion | ~5MB | ✅ Yes |
| **SQLite** | 3.35+ | Database | ~2MB | ✅ Yes |
| **Caddy** | 2.7+ | HTTPS reverse proxy | ~40MB | ⚠️ Optional |

**Total Required:** ~180MB
**Total with Caddy:** ~220MB

---

## Installation by Platform

### macOS (Development)

**Homebrew (recommended):**
```bash
# Install all required tools at once
brew install ffmpeg mediamtx ghostscript webp sqlite

# Optional: HTTPS support
brew install caddy
```

**Verify installation:**
```bash
ffmpeg -version          # Should show 6.0+
mediamtx --version       # Should show 1.5+
gs --version             # Should show 10.0+
cwebp -version           # Should show 1.3+
sqlite3 --version        # Should show 3.35+
```

---

### Ubuntu/Debian (Production)

```bash
# Update package list
sudo apt-get update

# Install all required tools
sudo apt-get install -y \
  ffmpeg \
  ghostscript \
  webp \
  sqlite3

# Install MediaMTX manually (not in apt)
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
sudo chmod +x /usr/local/bin/mediamtx

# Optional: Install Caddy
sudo apt-get install -y caddy
```

**Verify:**
```bash
which ffmpeg ghostscript cwebp mediamtx sqlite3
```

---

### Alpine Linux (Docker)

Already configured in `docker/Dockerfile`:

```dockerfile
RUN apk add --no-cache \
    ffmpeg \
    ffmpeg-libs \
    ghostscript \
    libwebp-tools \
    sqlite \
    sqlite-libs
```

MediaMTX is included as a separate Docker service in `docker-compose.yml`.

---

### Red Hat/CentOS/Fedora

```bash
# Enable EPEL repository (for FFmpeg)
sudo yum install -y epel-release

# Install tools
sudo yum install -y \
  ffmpeg \
  ghostscript \
  libwebp-tools \
  sqlite

# Install MediaMTX manually
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
```

---

### Arch Linux

```bash
sudo pacman -S ffmpeg ghostscript libwebp sqlite

# MediaMTX (from AUR)
yay -S mediamtx
```

---

## Detailed Tool Information

### 1. FFmpeg

**Purpose:** Video streaming, transcoding, and HLS generation

**Used for:**
- RTMP stream ingestion
- Video transcoding to HLS format
- Thumbnail generation from videos
- Audio/video format conversion

**Key features used:**
- H.264 video encoding
- AAC audio encoding
- HLS segmentation
- Hardware acceleration support

**Configuration:**
- Default installation is sufficient
- Hardware encoders (VideoToolbox, NVENC) are auto-detected

**Verification:**
```bash
ffmpeg -version
ffmpeg -encoders | grep h264
```

---

### 2. MediaMTX

**Purpose:** RTMP/HLS/WebRTC streaming server

**Used for:**
- RTMP stream ingestion from OBS/FFmpeg
- HLS output generation
- WebRTC ultra-low-latency streaming
- Stream recording
- Authentication hooks

**Configuration:**
- Configured via `mediamtx.yml`
- Integrates with Rust server for auth

**Ports:**
- 1935: RTMP ingestion
- 8888: HLS output
- 8889: WebRTC
- 9997: API
- 9998: Metrics

**Verification:**
```bash
mediamtx --version
curl http://localhost:9997/v3/paths/list
```

**Documentation:**
- https://github.com/bluenviron/mediamtx
- Project: `mediamtx.yml` configuration file

---

### 3. Ghostscript

**Purpose:** PDF rendering for thumbnail generation

**Used for:**
- Rendering first page of PDF documents to PNG
- High-quality PDF rasterization at 150 DPI
- Automatic background thumbnail generation

**Features:**
- Renders PDF first page to PNG
- 150 DPI for crisp thumbnails
- Anti-aliasing for smooth graphics

**Alternative:** None - Ghostscript is the industry standard

**Configuration:**
- No configuration needed
- Called via `tokio::process::Command`

**Verification:**
```bash
gs --version
gs -h
```

**Documentation:**
- https://www.ghostscript.com/
- Project: `docs/PDF_THUMBNAILS.md`
- Code: `crates/media-manager/src/pdf_thumbnail.rs`

---

### 4. WebP Tools

**Purpose:** PNG → WebP image conversion

**Used for:**
- Converting PNG thumbnails to WebP format
- Image optimization (87% smaller files)
- Lossless WebP encoding

**Key tool:** `cwebp` (command-line encoder)

**Benefits:**
- 87% smaller file sizes vs PNG
- Lossless compression
- Fast encoding
- Browser compatibility

**Configuration:**
- Quality: 85 (configurable in scripts)
- Lossless mode for thumbnails

**Verification:**
```bash
cwebp -version
cwebp -h
```

**Documentation:**
- https://developers.google.com/speed/webp
- Project: `docs/PDF_THUMBNAILS.md`

---

### 5. SQLite

**Purpose:** Database for media metadata

**Used for:**
- Media items (videos, images, documents)
- Access codes
- User sessions
- Tags and categories

**Features:**
- Embedded database (no server needed)
- ACID transactions
- Full-text search
- JSON support

**File:** `media.db`

**Schema:**
- See `migrations/` folder
- Auto-applied on startup

**Verification:**
```bash
sqlite3 --version
sqlite3 media.db ".tables"
```

---

### 6. Caddy (Optional)

**Purpose:** HTTPS reverse proxy

**Used for:**
- Automatic HTTPS with Let's Encrypt
- Reverse proxy to Rust server
- Static file serving
- WebSocket support

**Configuration:**
- See `Caddyfile` in project root
- Auto-configured for `app.appkask.com`

**Verification:**
```bash
caddy version
caddy validate --config Caddyfile
```

---

## Troubleshooting

### Command Not Found

**Problem:** `bash: gs: command not found`

**Solution:**
```bash
# macOS
brew install ghostscript

# Linux
sudo apt-get install ghostscript
```

### Wrong Version

**Problem:** Old version installed

**Solution:**
```bash
# macOS
brew upgrade ghostscript ffmpeg webp

# Linux
sudo apt-get update
sudo apt-get upgrade ghostscript ffmpeg webp
```

### Permission Denied

**Problem:** Cannot execute tool

**Solution:**
```bash
# Check permissions
which gs
ls -l $(which gs)

# Fix if needed
sudo chmod +x /usr/local/bin/gs
```

### Missing Libraries

**Problem:** `error while loading shared libraries`

**Solution (Linux):**
```bash
# Update library cache
sudo ldconfig

# Install missing libs
sudo apt-get install -y libwebp7 libgs9
```

---

## Docker

All system dependencies are included in the Docker images:

### Builder Stage (Alpine)
```dockerfile
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    nodejs \
    npm
```

### Runtime Stage (Alpine)
```dockerfile
RUN apk add --no-cache \
    ffmpeg \
    ffmpeg-libs \
    ghostscript \
    libwebp-tools \
    sqlite \
    sqlite-libs \
    ca-certificates \
    tzdata \
    curl \
    wget
```

### MediaMTX Container
- Uses official `bluenviron/mediamtx` image
- No additional dependencies needed

**Build:**
```bash
docker build -t video-server-rs -f docker/Dockerfile .
```

**Verify:**
```bash
docker run video-server-rs sh -c "gs --version && cwebp -version"
```

---

## CI/CD Considerations

### GitHub Actions Example

```yaml
- name: Install system dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y ffmpeg ghostscript webp sqlite3

- name: Install MediaMTX
  run: |
    wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
    tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
    sudo mv mediamtx /usr/local/bin/
```

---

## Security Considerations

### Ghostscript

**Sandboxing:** Always run with `-dSAFER` flag (enabled in code)

```rust
Command::new("gs")
    .args(["-dSAFER", "-dBATCH", "-dNOPAUSE"])
    // ...
```

**Why:** Prevents malicious PDFs from executing arbitrary code

### FFmpeg

**Input validation:** Always validate file paths and formats

**Resource limits:** Consider using `-t` flag to limit processing time

### WebP

**Memory limits:** WebP encoder has built-in safety limits

---

## Performance Notes

### Ghostscript
- **CPU:** Moderate (200-500ms per PDF page)
- **Memory:** ~10-20MB per document
- **Disk I/O:** Minimal (small temp PNG)

### FFmpeg
- **CPU:** High during transcoding
- **Memory:** Depends on video resolution
- **Disk I/O:** High during HLS generation

### WebP
- **CPU:** Low (fast PNG → WebP)
- **Memory:** Minimal
- **Disk I/O:** Minimal

---

## Version Compatibility

### Minimum Versions

| Tool | Minimum | Recommended | Latest Tested |
|------|---------|-------------|---------------|
| FFmpeg | 4.0 | 6.0+ | 6.1 |
| MediaMTX | 1.0 | 1.5+ | 1.5.1 |
| Ghostscript | 9.50 | 10.0+ | 10.06 |
| WebP | 1.0 | 1.3+ | 1.6 |
| SQLite | 3.35 | 3.40+ | 3.45 |

### Breaking Changes

- **FFmpeg 5.0+:** Changed some command-line flags
- **Ghostscript 10.0+:** Improved security defaults
- **MediaMTX 1.0+:** New API structure

---

## Alternative Tools (Not Recommended)

| Instead of | Don't use | Why not |
|------------|-----------|---------|
| Ghostscript | ImageMagick PDF | Slower, security issues |
| Ghostscript | Poppler | Additional dependency |
| WebP tools | ImageMagick WebP | Larger binary |
| FFmpeg | GStreamer | More complex setup |

---

## License Information

All dependencies are open-source:

- **FFmpeg:** LGPL 2.1+
- **MediaMTX:** MIT
- **Ghostscript:** AGPL 3.0 (GPL for embedded use)
- **WebP:** BSD-style
- **SQLite:** Public domain
- **Caddy:** Apache 2.0

---

## Support

**Installation issues:** Check platform-specific package manager docs

**Missing features:** Ensure latest version is installed

**Performance issues:** See Performance Notes section above

**Security concerns:** Keep all tools updated via package manager

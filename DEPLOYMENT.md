# Deployment Guide

**Last Updated:** January 2026

This guide covers deploying the media server to production or staging environments.

---

## 📋 Table of Contents

- [Quick Deploy](#quick-deploy)
- [Prerequisites](#prerequisites)
- [Step-by-Step Deployment](#step-by-step-deployment)
- [CSS Build (Required)](#css-build-required)
- [Post-Deployment](#post-deployment)
- [Troubleshooting](#troubleshooting)
- [Production Checklist](#production-checklist)

---

## 🚀 Quick Deploy

### Option 1: Docker (Easiest)

```bash
# 1. Pull latest code
git pull origin main

# 2. Start with Docker Compose
cd docker
docker-compose up -d

# 3. View logs
docker-compose logs -f

# Access at http://localhost:3000
```

**See:** `docker/README.md` for complete Docker documentation.

---

### Option 2: Using Build Script (Native)

```bash
# 1. Pull latest code
git pull origin main

# 2. Run build script (builds CSS + Rust)
./scripts/admin/build.sh --release

# 3. Run migrations (if any)
# sqlx migrate run

# 4. Start the server
./target/release/video-server-rs
```

### Option 3: Manual Build (Native)

```bash
# 1. Pull latest code
git pull origin main

# 2. Build CSS (REQUIRED - not in git)
npm install
npm run build:css

# 3. Build Rust application
cargo build --release

# 4. Run migrations (if any)
# sqlx migrate run

# 5. Start the server
./target/release/video-server-rs
```

**Note:** The build script (`scripts/run/build.sh`) automates steps 2-3 and includes verification.

**Docker vs Native:**
- Docker: Easiest, includes all dependencies, isolated environment
- Native: More control, direct access to system, no Docker required

---

## ✅ Prerequisites

### System Requirements
- **Rust:** 1.70+ (stable)
- **Node.js:** 18+ (for CSS build)
- **FFmpeg:** Latest version with H.264 support
- **MediaMTX:** Latest version (if using live streaming)
- **SQLite:** 3.35+ (usually included)

### Install Prerequisites

**Ubuntu/Debian:**
```bash
# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 18+
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# FFmpeg
sudo apt-get install -y ffmpeg

# MediaMTX (optional, for live streaming)
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
```

**macOS:**
```bash
# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js
brew install node

# FFmpeg
brew install ffmpeg

# MediaMTX (optional)
brew install mediamtx
```

---

## 📦 Step-by-Step Deployment

### 1. Clone or Update Repository

**First time:**
```bash
git clone https://github.com/yourusername/video-server-rs_v1.git
cd video-server-rs_v1
```

**Updating:**
```bash
cd video-server-rs_v1
git pull origin main
```

### 2. Build CSS (REQUIRED) ⚠️

**The CSS file is NOT in git** (it's in `.gitignore`). You must build it:

```bash
# Install Node dependencies
npm install

# Build CSS (production - minified)
npm run build:css
```

**Output:**
```
static/css/tailwind.css  (~90KB minified)
```

**Verify it was created:**
```bash
ls -lh static/css/tailwind.css
```

**Development mode** (with file watching):
```bash
npm run watch:css  # Rebuilds on file changes
```

### 3. Build Rust Application

**Development build:**
```bash
cargo build
```

**Production build** (optimized, recommended):
```bash
cargo build --release
```

The binary will be at:
- Development: `target/debug/video-server-rs`
- Release: `target/release/video-server-rs`

### 4. Setup Storage Directories

```bash
# Create storage structure
mkdir -p storage/images
mkdir -p storage/videos
mkdir -p storage/temp

# Set permissions (if needed)
chmod 755 storage
chmod 755 storage/images storage/videos storage/temp
```

### 5. Configure Environment

Create `.env` file if needed:

```bash
cat > .env << 'EOF'
# Database
DATABASE_URL=sqlite:video.db

# Server
RUST_LOG=info
PORT=3000

# Optional: OIDC Configuration
# OIDC_ISSUER_URL=https://your-oidc-provider.com
# OIDC_CLIENT_ID=your-client-id
# OIDC_CLIENT_SECRET=your-client-secret
# OIDC_REDIRECT_URL=https://yourdomain.com/auth/callback

# Optional: Emergency Login (disable in production!)
# ENABLE_EMERGENCY_LOGIN=false
# SU_USER=admin
# SU_PWD=secure-password-here
EOF
```

### 6. Run Database Migrations (if any)

```bash
# If using sqlx migrations
sqlx migrate run

# Or initialize database with script
./scripts/dev/init-database.sh
```

### 7. Start the Server

**Development:**
```bash
cargo run
```

**Production:**
```bash
./target/release/video-server-rs
```

**As a service** (see systemd section below)

---

## 🎨 CSS Build (Required)

### Why CSS Build is Required

The Tailwind CSS file is **generated from templates** and **NOT committed to git**. You must build it on every deployment.

### CSS Build Commands

```bash
# Production build (minified)
npm run build:css

# Development build (with watch mode)
npm run watch:css

# Check if CSS exists
ls -lh static/css/tailwind.css
```

### What Gets Generated

```
static/css/
├── input.css          # Source (in git)
└── tailwind.css       # Generated (NOT in git) ⚠️
```

### If CSS is Missing

**Symptoms:**
- No styling on web pages
- Pages look broken/plain HTML
- Browser console shows 404 for `/static/css/tailwind.css`

**Fix:**
```bash
npm install
npm run build:css
```

### CSS Configuration Files

The build uses these config files (committed to git):
- `tailwind.config.ts` - Tailwind configuration
- `postcss.config.cjs` - PostCSS configuration
- `package.json` - Build scripts
- `static/css/input.css` - Source CSS with Tailwind directives

---

## 🔧 Post-Deployment

### Verify Installation

```bash
# Check server is running
curl http://localhost:3000/health

# Check static files are served
curl -I http://localhost:3000/static/css/tailwind.css

# Check FFmpeg
ffmpeg -version

# Check database
sqlite3 video.db "SELECT COUNT(*) FROM videos;"
```

### Generate Admin Thumbnails (if needed)

```bash
cargo run --bin generate-thumbnails
```

### Test Video Upload

1. Navigate to: `http://localhost:3000/videos/new`
2. Upload a test video
3. Check transcoding progress
4. Verify playback works

---

## 🐧 Running as a Systemd Service

Create `/etc/systemd/system/video-server.service`:

```ini
[Unit]
Description=Video Server (Rust + MediaMTX)
After=network.target

[Service]
Type=simple
User=your-username
WorkingDirectory=/path/to/video-server-rs_v1
Environment="RUST_LOG=info"
ExecStart=/path/to/video-server-rs_v1/target/release/video-server-rs
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable video-server
sudo systemctl start video-server
sudo systemctl status video-server
```

**View logs:**
```bash
sudo journalctl -u video-server -f
```

---

## 🔥 Troubleshooting

### CSS Not Loading (Most Common Issue)

**Problem:** Pages have no styling, look like plain HTML

**Solution:**
```bash
# Build the CSS
npm install
npm run build:css

# Verify it exists
ls -lh static/css/tailwind.css

# Restart server
```

### Port Already in Use

**Problem:** `Address already in use (os error 98)`

**Solution:**
```bash
# Find what's using port 3000
lsof -i :3000

# Kill the process
kill -9 <PID>

# Or use a different port
PORT=3001 cargo run
```

### FFmpeg Not Found

**Problem:** Video transcoding fails

**Solution:**
```bash
# Install FFmpeg
sudo apt-get install ffmpeg

# Verify
ffmpeg -version
which ffmpeg
```

### Permission Denied on Storage

**Problem:** Cannot write to storage directory

**Solution:**
```bash
# Fix permissions
chmod -R 755 storage/
chown -R your-username:your-group storage/
```

### Database Locked

**Problem:** `database is locked`

**Solution:**
```bash
# Stop all instances
pkill video-server-rs

# Check database isn't corrupted
sqlite3 video.db "PRAGMA integrity_check;"

# Restart server
```

### Node Modules Missing

**Problem:** `npm run build:css` fails

**Solution:**
```bash
# Remove node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
npm run build:css
```

---

## ✅ Production Checklist

Before deploying to production:

### Security
- [ ] Change all default passwords
- [ ] Disable emergency login (`ENABLE_EMERGENCY_LOGIN=false`)
- [ ] Set up HTTPS (use reverse proxy like Caddy/Nginx)
- [ ] Configure firewall (allow only necessary ports)
- [ ] Set secure session secret
- [ ] Review CORS origins
- [ ] Enable rate limiting (if implemented)

### Configuration
- [ ] Set `RUST_LOG=info` (not `debug` or `trace`)
- [ ] Configure OIDC authentication
- [ ] Set proper `DATABASE_URL`
- [ ] Configure storage paths
- [ ] Set up backup strategy

### Performance
- [ ] Use `--release` build
- [ ] Build CSS with `--minify`
- [ ] Configure CDN for static assets (optional)
- [ ] Set up database backups
- [ ] Monitor disk space (videos use lots of space!)

### Monitoring
- [ ] Set up log rotation
- [ ] Configure system monitoring
- [ ] Set up alerts for errors
- [ ] Monitor disk space
- [ ] Track video processing queue

### Testing
- [ ] Test video upload and playback
- [ ] Test image upload and serving
- [ ] Test access codes
- [ ] Test group permissions
- [ ] Verify CSS loads correctly
- [ ] Check mobile responsiveness

---

## 🔄 Update Procedure

For updating an existing deployment:

### Option 1: Using Build Script (Recommended)

```bash
# 1. Stop the server
sudo systemctl stop video-server

# 2. Backup database
cp video.db video.db.backup-$(date +%Y%m%d-%H%M%S)

# 3. Pull latest code
git pull origin main

# 4. Build everything
./scripts/admin/build.sh --release

# 5. Run migrations (if any)
# sqlx migrate run

# 6. Restart server
sudo systemctl start video-server

# 7. Check logs
sudo journalctl -u video-server -f
```

### Option 2: Manual Update

```bash
# 1. Stop the server
sudo systemctl stop video-server

# 2. Backup database
cp video.db video.db.backup-$(date +%Y%m%d-%H%M%S)

# 3. Pull latest code
git pull origin main

# 4. Rebuild CSS (REQUIRED)
npm install
npm run build:css

# 5. Build Rust
cargo build --release

# 6. Run migrations (if any)
# sqlx migrate run

# 7. Restart server
sudo systemctl start video-server

# 8. Check logs
sudo journalctl -u video-server -f
```

---

## 📞 Support

If you encounter issues:

1. Check the logs: `sudo journalctl -u video-server -f`
2. Review `TROUBLESHOOTING.md`
3. Check GitHub issues
4. Verify all prerequisites are installed
5. Ensure CSS was built (`npm run build:css`)

---

**Remember:** The CSS file is NOT in git and MUST be built on every deployment!

```bash
# Quick way: Use the build script
./scripts/admin/build.sh --release

# Or manually:
npm install && npm run build:css
```

**See also:** `scripts/README.md` for build script documentation

---

**Document Version:** 1.0  
**Last Updated:** January 2026  
**For Questions:** Check documentation in `docs/` directory
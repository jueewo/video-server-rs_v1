# Scripts Directory

This directory contains utility scripts organized by audience:

- **`user/`** - Scripts for end users to prepare and manage media
- **`run/`** - Scripts for running/deploying the server (build, Docker)
- **`admin/`** - Scripts for server administrators (maintenance, utilities)
- **`dev/`** - Scripts for developers (setup, testing, debugging)

---

## 👥 User Scripts

### 📹 `user/prepare-video.sh`

**Purpose:** Offline video preparation tool - transcodes videos to HLS format with multiple quality variants, exactly matching the server's upload pipeline.

**Usage:**
```bash
./scripts/user/prepare-video.sh <input-video> <slug> [public|private]
```

**Example:**
```bash
./scripts/user/prepare-video.sh my-video.mp4 my-awesome-video public
```

**What it does:**
- Transcodes video to HLS format with multiple quality variants (1080p, 720p, 480p, 360p)
- Creates master playlist and segment files
- Generates thumbnail and poster images
- Organizes output in server-ready directory structure
- Validates with FFmpeg to ensure compatibility

**Output Structure:**
```
storage/videos/public/my-awesome-video/
├── hls/
│   ├── master.m3u8
│   ├── 1080p/
│   │   ├── index.m3u8
│   │   ├── segment_000.ts
│   │   ├── segment_001.ts
│   │   └── ...
│   ├── 720p/
│   ├── 480p/
│   └── 360p/
├── thumbnail.jpg
└── poster.jpg
```

**After running:**
1. Start the server: `cargo run`
2. Navigate to: `http://localhost:3000/videos/new`
3. Select the prepared folder slug (e.g., `my-awesome-video`)
4. Fill in metadata and click "Register Video"

**Requirements:**
- FFmpeg with H.264 (libx264) support
- Sufficient disk space (output is ~70-80% of input size)
- Input video in any FFmpeg-supported format

**Benefits:**
- Process large videos offline without blocking the server
- Consistent quality settings matching server pipeline
- Faster registration in the UI (transcoding already done)
- Batch processing capability

---

## 🚀 Run Scripts

### 🏗️ `run/build.sh`

**Purpose:** Complete build script for server deployment - builds CSS and Rust in one command.

**Usage:**
```bash
# Full production build (recommended for deployment)
./scripts/run/build.sh

# Development build (faster)
./scripts/run/build.sh --dev

# Only rebuild CSS (after template changes)
./scripts/run/build.sh --css-only

# Only rebuild Rust (after code changes)
./scripts/run/build.sh --rust-only

# Clean build (remove all artifacts first)
./scripts/run/build.sh --clean --release
```

**What it does:**
- Checks prerequisites (Node.js, Rust, FFmpeg)
- Installs Node dependencies
- Builds Tailwind CSS (REQUIRED - not in git)
- Compiles Rust application (debug or release)
- Verifies all build artifacts exist
- Creates storage directories if missing
- Shows helpful next steps

**Options:**
- `--dev` - Development build (faster, larger binary)
- `--release` - Production build (default, optimized)
- `--css-only` - Only build CSS, skip Rust
- `--rust-only` - Only build Rust, skip CSS
- `--clean` - Clean artifacts before building
- `--help` - Show usage information

**When to use:**
- Every deployment (CSS must be rebuilt)
- After pulling code updates
- When CSS is broken/missing
- Setting up new server
- After modifying templates

**Benefits:**
- Single command for complete build
- Automatic prerequisite checking
- Colored output with progress indicators
- Build verification and error detection
- Helpful error messages and next steps

**Output:**
- `static/css/tailwind.css` (~90KB minified)
- `target/release/video-server-rs` (production)
- `target/debug/video-server-rs` (development)

---

### 🐳 Docker Deployment

**Purpose:** Run the entire stack with Docker and Docker Compose.

**Quick Start:**
```bash
# Start everything (media-server + MediaMTX)
docker-compose up -d

# View logs
docker-compose logs -f

# Stop everything
docker-compose down
```

**Services:**
- `media-server` - Rust application (port 3000)
- `mediamtx` - Streaming server (ports 1935, 8888, 8889, 9997, 9998)

**Benefits:**
- Separate services for better architecture
- Easy scaling and maintenance
- Built-in health checks
- Isolated networking
- Volume persistence

**See:** `DOCKER.md` for complete Docker documentation

---

## 🔧 Admin Scripts

### 🖼️ `admin/generate_thumbnails.rs`

**Purpose:** Maintenance tool to regenerate thumbnails for all images in the database.

**Usage:**
```bash
cargo run --bin generate-thumbnails
```

**What it does:**
- Connects to the database and finds all images
- Generates 400x400 thumbnails maintaining aspect ratio
- Saves thumbnails as WebP format
- Skips SVG files (vector format)
- Provides detailed progress and error reporting

**When to use:**
- After bulk image imports
- When thumbnail generation failed during upload
- After changing thumbnail size settings
- Database migration or recovery

**Requirements:**
- Database must be accessible (video.db)
- Original images must exist in storage directory
- Write permissions to storage directory

**Output:**
- Creates `{slug}_thumb.webp` for each image
- Shows progress and summary statistics

---

## 🛠️ Developer Scripts

Developer scripts are located in `dev/` and include:

- **Setup scripts** - Database initialization, storage setup
- **Test scripts** - Access codes, emergency login, images, HLS
- **Utilities** - Migration tools, transcoding

See `dev/` directory for the full list of developer tools.

---

## 🚀 Quick Start for Users

### Preparing Your First Video

```bash
# 1. Make sure FFmpeg is installed
ffmpeg -version

# 2. Prepare your video
./scripts/user/prepare-video.sh ~/Downloads/my-video.mp4 welcome-video public

# 3. Start the server (if not already running)
cargo run

# 4. Register in the UI
# Visit: http://localhost:3000/videos/new
# Select folder: welcome-video
# Fill metadata and save
```

### Batch Processing Multiple Videos

```bash
# Create a simple loop for multiple videos
for video in ~/Videos/*.mp4; do
  filename=$(basename "$video" .mp4)
  slug=$(echo "$filename" | tr '[:upper:]' '[:lower:]' | tr ' ' '-')
  ./scripts/user/prepare-video.sh "$video" "$slug" public
done
```

---

## 📝 Script Guidelines

### Using User Scripts

User scripts are designed to be:
- ✅ **Self-contained** - Include all necessary dependencies
- ✅ **Well-documented** - Clear usage instructions and examples
- ✅ **Validated** - Check prerequisites before running
- ✅ **User-friendly** - Colorized output and progress indicators
- ✅ **Safe** - Validate inputs and handle errors gracefully

### Adding New User Scripts

When creating user-facing scripts:

1. **Place in `user/` directory:**
   ```bash
   touch scripts/user/new-tool.sh
   chmod +x scripts/user/new-tool.sh
   ```

2. **Add comprehensive header:**
   ```bash
   #!/bin/bash
   #==============================================================================
   # new-tool.sh - Brief Description
   #==============================================================================
   #
   # Detailed explanation of what this script does
   #
   # Usage:
   #   ./new-tool.sh <required-arg> [optional-arg]
   #
   # Example:
   #   ./new-tool.sh input.txt output.txt
   #
   #==============================================================================
   
   set -e  # Exit on error
   ```

3. **Include validation:**
   - Check prerequisites (commands, files)
   - Validate input parameters
   - Provide helpful error messages

4. **Add progress indicators:**
   - Use colored output (green ✓, red ✗, yellow ⚠)
   - Show step-by-step progress
   - Display summary at completion

5. **Update this README:**
   - Add to "User Scripts" section
   - Document usage with examples
   - Explain requirements and output

6. **Link in main documentation:**
   - Add to `README.md`
   - Add to `MASTER_PLAN.md`
   - Update `QUICKSTART.md` if relevant

---

## 🔍 Troubleshooting

### User Script Issues

**Problem:** Script won't run - permission denied
```bash
chmod +x scripts/user/prepare-video.sh
```

**Problem:** FFmpeg not found
```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt-get install ffmpeg

# Verify
ffmpeg -version
```

**Problem:** "libx264 encoder not found"
```bash
# Need FFmpeg with H.264 support
# macOS (brew version includes it)
brew reinstall ffmpeg

# Linux - install with x264
sudo apt-get install ffmpeg libx264-dev
```

**Problem:** Output directory already exists
```bash
# The script will fail if output exists to prevent overwriting
# Remove the existing directory or choose a different slug:
rm -rf storage/videos/public/existing-slug
```

**Problem:** Out of disk space
```bash
# Check available space
df -h storage/

# Clean up old videos
rm -rf storage/videos/*/old-video-slug/
```

---

## 📚 Related Documentation

### For Users
- **Main README:** `../README.md` - Getting started guide
- **Quickstart:** `../QUICKSTART.md` - Fast setup
- **Master Plan:** `../MASTER_PLAN.md` - Complete feature overview

### For Developers
- **Setup & Testing:** `dev/` directory - Developer tools
- **Architecture:** `../docs/architecture/` - System design
- **Features:** `../docs/features/` - Feature documentation

---

## 🤝 Contributing

### User Script Priorities

When considering new user scripts, prioritize:

1. **Common workflows** - Tasks users do frequently
2. **Time-saving tools** - Automate manual processes
3. **Batch operations** - Process multiple items
4. **Quality of life** - Make complex tasks simple
5. **Error prevention** - Validate before execution

### Examples of Good User Scripts

- ✅ Batch video preparation
- ✅ Media validation tools
- ✅ Thumbnail generation
- ✅ Metadata extraction
- ✅ Storage cleanup utilities

### Keep Developer Scripts Separate

Developer scripts belong in `dev/` and include:
- Database migrations
- Test suites
- Debug utilities
- Performance profiling
- Development setup

---

**Directory Structure:**
```
scripts/
├── README.md          # This file
├── user/              # User-facing scripts
│   └── prepare-video.sh
├── run/               # Running/deployment scripts
│   └── build.sh       # Deployment build script ⭐
├── admin/             # Admin/maintenance scripts
│   └── generate_thumbnails.rs
└── dev/               # Developer scripts
    ├── init-database.sh
    ├── setup-images.sh
    ├── test-*.sh
    └── ...
```

**Last Updated:** January 2026  
**User Scripts:** 1  
**Run Scripts:** 1  
**Admin Scripts:** 1  
**Dev Scripts:** 12

**Quick Deploy:**
```bash
# Native deployment
./scripts/run/build.sh --release

# Docker deployment
docker-compose up -d
```
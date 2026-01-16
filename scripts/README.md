# Scripts Directory

This directory contains utility scripts for setup, testing, and maintenance of the video server.

## 📋 Available Scripts

### 🔧 Setup & Initialization

#### `init-database.sh`
**Purpose:** Initialize the SQLite database with required tables

**Usage:**
```bash
./scripts/init-database.sh
```

**What it does:**
- Creates or recreates the SQLite database
- Sets up tables for videos, images, and users
- Initializes schema for the application

**When to use:**
- First time setup
- After database corruption
- When resetting the database

---

#### `setup-images.sh`
**Purpose:** Set up directory structure for image storage

**Usage:**
```bash
./scripts/setup-images.sh
```

**What it does:**
- Creates `storage/images/` directory
- Sets proper permissions
- Prepares image storage locations

**When to use:**
- First time setup
- After storage directory issues
- When deploying to new server

---

#### `setup_db.sh`
**Purpose:** Database setup utility (legacy)

**Usage:**
```bash
./scripts/setup_db.sh
```

**Note:** Consider using `init-database.sh` for newer setups.

---

### 🧪 Testing Scripts

#### `test-emergency-login.sh`
**Purpose:** Test the emergency login feature

**Usage:**
```bash
./scripts/test-emergency-login.sh
```

**What it tests:**
- Emergency login route availability
- Login form structure
- Invalid credentials handling
- Configuration check

**Requirements:**
- Server must be running on port 3000
- Emergency login must be enabled in `.env`

**Output:**
- ✅ Green: Tests passed
- ⚠️  Yellow: Warnings/info
- ❌ Red: Tests failed

---

#### `test-images.sh`
**Purpose:** Test image upload and serving functionality

**Usage:**
```bash
./scripts/test-images.sh
```

**What it tests:**
- Image upload endpoint
- Image serving/retrieval
- Authentication requirements
- Error handling

**Requirements:**
- Server must be running
- Valid authentication session

---

#### `test-hls.html`
**Purpose:** HTML test page for HLS/WebRTC streaming

**Usage:**
1. Start MediaMTX and the server
2. Open in browser: `http://localhost:3000/test`
3. Or open directly: `file:///.../scripts/test-hls.html`

**Features:**
- HLS playback testing
- WebRTC low-latency testing
- Stream quality monitoring
- Player controls and debugging

---

#### `test-access-codes.sh`
**Purpose:** Test access codes functionality for sharing media

**Usage:**
```bash
./scripts/test-access-codes.sh
```

**What it tests:**
- Creating access codes with multiple media items
- Listing access codes
- Accessing videos and images with access codes
- Expiration date validation
- Cleanup of test codes

**Requirements:**
- Server must be running on port 3000
- Database must be initialized
- Test media items must exist (welcome video, logo image)

**Output:**
- ✅ Green: Tests passed
- ❌ Red: Tests failed
- Detailed error messages for debugging

---

### 📺 Streaming Scripts

#### `live_streaming_on_macbook.sh`
**Purpose:** Quick script to start streaming from macOS

**Usage:**
```bash
./scripts/live_streaming_on_macbook.sh
```

**What it does:**
- Detects available cameras and microphones
- Starts FFmpeg streaming to RTMP endpoint
- Configures optimal settings for macOS

**Requirements:**
- FFmpeg installed
- Camera and microphone permissions
- MediaMTX running

---

#### `transcode.sh`
**Purpose:** Video transcoding utility

**Usage:**
```bash
./scripts/transcode.sh input.mp4 output.mp4
```

**What it does:**
- Transcodes video files
- Optimizes for streaming
- Configures codecs and bitrates

---

## 🚀 Quick Start Workflow

### First Time Setup

```bash
# 1. Initialize database
./scripts/init-database.sh

# 2. Setup image storage
./scripts/setup-images.sh

# 3. Start the server
cargo run

# 4. Test emergency login (if enabled)
./scripts/test-emergency-login.sh

# 5. Test image functionality
./scripts/test-images.sh
```

### Daily Development

```bash
# Start streaming test
./scripts/live_streaming_on_macbook.sh

# Open test page
# Visit: http://localhost:3000/test
```

---

## 📝 Script Guidelines

### Adding New Scripts

1. **Create the script:**
   ```bash
   touch scripts/new-script.sh
   chmod +x scripts/new-script.sh
   ```

2. **Add header:**
   ```bash
   #!/bin/bash
   # Description of what this script does
   set -e
   ```

3. **Update this README:**
   - Add script to appropriate section
   - Document usage and requirements
   - Explain what it does

### Best Practices

- ✅ Make scripts executable: `chmod +x`
- ✅ Use `set -e` for error handling
- ✅ Add help text with `--help` flag
- ✅ Print clear messages during execution
- ✅ Check prerequisites before running
- ✅ Provide useful error messages

---

## 🔍 Troubleshooting

### Script Won't Run

**Problem:** Permission denied
```bash
chmod +x scripts/script-name.sh
```

**Problem:** Command not found
```bash
# Run from project root:
./scripts/script-name.sh
```

### Tests Failing

**Emergency Login Tests:**
- Check server is running: `lsof -i :3000`
- Verify `ENABLE_EMERGENCY_LOGIN=true` in `.env`
- Check credentials are set: `SU_USER` and `SU_PWD`

**Image Tests:**
- Ensure server is running
- Check authentication is configured
- Verify storage directory exists

**Streaming Tests:**
- Confirm MediaMTX is running: `lsof -i :1935`
- Check FFmpeg is installed: `ffmpeg -version`
- Verify camera/mic permissions on macOS

---

## 📚 Related Documentation

- **Main Documentation:** `../docs/README.md`
- **Emergency Login:** `../docs/auth/EMERGENCY_LOGIN_QUICKSTART.md`
- **Live Streaming:** `../docs/LIVE_STREAMING_GUIDE.md`
- **Image Serving:** `../docs/features/IMAGE_SERVING.md`

---

## 🤝 Contributing

When adding new scripts:

1. Follow naming conventions (lowercase with hyphens)
2. Add proper documentation in this README
3. Include error handling and user feedback
4. Test on both macOS and Linux if possible
5. Update related documentation

---

**Last Updated:** January 2024  
**Total Scripts:** 9
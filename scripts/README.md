# Scripts Directory

**Purpose:** Command-line utilities and automation scripts for the media server  
**Last Updated:** February 2026  
**Status:** ✅ Organized and Ready to Use

---

## 📂 Directory Structure

```
scripts/
├── admin/          👑 Deployment & administration
├── dev/            🔧 Development utilities
├── maintenance/    🔨 Cleanup & maintenance
├── testing/        🧪 API & feature testing
├── run/            🚀 Runtime & build scripts
└── user/           👤 User helper scripts
```

---

## 👑 admin/ - Deployment & Administration

**Purpose:** Production deployment, system management, documentation tools

### Scripts
- **`deploy-production.sh`** - Deploy to production server
  - Builds, transfers, and restarts application
  - Production deployment workflow
  
- **`mark_project_complete.sh`** - Mark project as production-ready
  - Updates status documents
  - Creates PROJECT_STATUS.md
  
- **`organize_docs.sh`** - Organize documentation files
  - Moves docs to appropriate folders
  - Creates navigation READMEs
  
- **`organize_scripts.sh`** - Organize script files
  - This script - organizes scripts into folders

### Usage
```bash
# Deploy to production
./scripts/admin/deploy-production.sh

# Update project status
./scripts/admin/mark_project_complete.sh

# Reorganize documentation
./scripts/admin/organize_docs.sh
```

---

## 🔧 dev/ - Development Utilities

**Purpose:** Development tools, database setup, testing helpers

### Scripts
- **`debug_media.sh`** - Debug media API responses
- **`init-database.sh`** - Initialize database with sample data
- **`live_streaming_on_macbook.sh`** - Start live stream on macOS
- **`migrate-storage.sh`** - Migrate storage structure
- **`setup-images.sh`** - Setup image serving functionality
- **`setup_db.sh`** - Quick database setup
- **`test-access-codes.sh`** - Test access code functionality
- **`test-emergency-login.sh`** - Test emergency login
- **`test-images.sh`** - Test image upload/serving
- **`test-tailwind-v4.sh`** - Test Tailwind CSS v4 build
- **`test_migrations.sh`** - Test database migrations
- **`transcode.sh`** - Transcode videos to HLS

### Usage
```bash
# Initialize development database
./scripts/dev/init-database.sh

# Test access codes
./scripts/dev/test-access-codes.sh

# Test emergency login
./scripts/dev/test-emergency-login.sh
```

---

## 🔨 maintenance/ - Cleanup & Maintenance

**Purpose:** System maintenance, cleanup, and update scripts

### Scripts
- **`deactivate_legacy_managers.sh`** - Deactivate legacy manager crates
- **`update_video_thumbnails.sh`** - Regenerate video thumbnails

### Usage
```bash
# Update all video thumbnails
./scripts/maintenance/update_video_thumbnails.sh

# Deactivate legacy managers (if needed)
./scripts/maintenance/deactivate_legacy_managers.sh
```

---

## 🧪 testing/ - API & Feature Testing

**Purpose:** Test API endpoints, features, and integration

### Scripts
- **`delete_media.sh`** - Interactive media deletion tool
- **`test_delete_manual.sh`** - Test manual deletion
- **`test_delete_one.sh`** - Test single item deletion
- **`test_json.sh`** - Test JSON API responses
- **`test_unified_upload.sh`** - Test unified upload API

### Usage
```bash
# Test unified upload
./scripts/testing/test_unified_upload.sh

# Interactive media deletion
./scripts/testing/delete_media.sh

# Test JSON responses
./scripts/testing/test_json.sh
```

---

## 🚀 run/ - Runtime & Build Scripts

**Purpose:** Build, compile, and run the application

### Scripts
- **`build.sh`** - Complete build process
  - Builds CSS
  - Compiles Rust
  - Verifies deployment readiness

### Usage
```bash
# Full production build
./scripts/run/build.sh

# Build with verbose output
./scripts/run/build.sh --verbose
```

---

## 👤 user/ - User Helper Scripts

**Purpose:** Helper scripts for content creators and users

### Scripts
- **`prepare-video.sh`** - Prepare videos for upload
  - Optimizes video for streaming
  - Generates thumbnails
  - Transcodes to appropriate format

### Usage
```bash
# Prepare a video for upload
./scripts/user/prepare-video.sh input.mp4

# With custom output
./scripts/user/prepare-video.sh input.mp4 output.mp4
```

---

## 🎯 Quick Reference

### By Use Case

**Setting up development environment**
```bash
./scripts/dev/init-database.sh
./scripts/dev/setup-images.sh
```

**Building for production**
```bash
./scripts/run/build.sh
./scripts/admin/deploy-production.sh
```

**Testing features**
```bash
./scripts/testing/test_unified_upload.sh
./scripts/testing/test_json.sh
./scripts/dev/test-access-codes.sh
```

**Maintaining system**
```bash
./scripts/maintenance/update_video_thumbnails.sh
```

**Preparing content**
```bash
./scripts/user/prepare-video.sh video.mp4
```

---

## 📋 Script Naming Convention

- **`test-*.sh`** - Testing scripts
- **`setup-*.sh`** - Setup/initialization scripts
- **`*-production.sh`** - Production-related scripts
- **`debug-*.sh`** - Debugging utilities
- **`organize-*.sh`** - Organization tools

---

## 🔒 Safety Notes

### Admin Scripts
- **Destructive operations:** Always backup before running admin scripts
- **Production impact:** Test in development first

### Maintenance Scripts
- **Data changes:** May modify database or files
- **Backup first:** Always backup before maintenance

### Testing Scripts
- **Safe to run:** Testing scripts don't modify production data
- **Development only:** Best run in development environment

---

## 📝 Adding New Scripts

When adding a new script:

1. **Choose the right folder:**
   - Admin → Deployment/management
   - Dev → Development tools
   - Maintenance → Cleanup/updates
   - Testing → API/feature tests
   - Run → Build/runtime
   - User → End-user helpers

2. **Make it executable:**
   ```bash
   chmod +x scripts/category/your-script.sh
   ```

3. **Add documentation:**
   - Update this README
   - Add usage comments in script
   - Document prerequisites

4. **Follow conventions:**
   - Use bash shebang: `#!/bin/bash`
   - Add error handling: `set -e`
   - Use descriptive names
   - Add help text: `--help`

---

## 🔍 Finding the Right Script

**I want to...**

- **Deploy to production** → `admin/deploy-production.sh`
- **Test the API** → `testing/test_*.sh`
- **Set up dev environment** → `dev/init-database.sh`
- **Build for release** → `run/build.sh`
- **Update thumbnails** → `maintenance/update_video_thumbnails.sh`
- **Prepare videos** → `user/prepare-video.sh`
- **Debug media** → `dev/debug_media.sh`
- **Test access codes** → `dev/test-access-codes.sh`

---

## 🛠️ Common Tasks

### First Time Setup
```bash
# 1. Initialize database
./scripts/dev/init-database.sh

# 2. Setup images
./scripts/dev/setup-images.sh

# 3. Build CSS
npm run build:css

# 4. Run server
cargo run
```

### Testing Workflow
```bash
# 1. Test uploads
./scripts/testing/test_unified_upload.sh

# 2. Test access codes
./scripts/dev/test-access-codes.sh

# 3. Test emergency login
./scripts/dev/test-emergency-login.sh
```

### Production Deployment
```bash
# 1. Build everything
./scripts/run/build.sh

# 2. Test locally
cargo test

# 3. Deploy
./scripts/admin/deploy-production.sh
```

---

## 💡 Tips

- **Always read script help:** Most scripts have `--help` option
- **Test in dev first:** Never run scripts in production without testing
- **Check prerequisites:** Some scripts require specific setup
- **Use absolute paths:** Or run from project root
- **Backup before maintenance:** Especially for database changes

---

## 📚 Related Documentation

- **[../QUICKSTART.md](../QUICKSTART.md)** - Quick setup guide
- **[../DEPLOYMENT.md](../DEPLOYMENT.md)** - Production deployment
- **[../docs/API_TESTING_GUIDE.md](../docs/API_TESTING_GUIDE.md)** - API testing
- **[../docs_dev/](../docs_dev/)** - Developer documentation

---

**Script Organization:** Complete  
**Total Scripts:** 20+ across 6 categories  
**Status:** ✅ Ready to Use  
**Last Organized:** February 2026
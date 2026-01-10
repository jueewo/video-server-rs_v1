# Video Storage Migration Guide

## Overview

The video storage structure has been reorganized for better clarity and consistency with the image storage system.

## What Changed

### Old Structure
```
storage/
├── public/              # Public videos (mixed with images)
│   ├── video1/
│   ├── video2/
│   └── images/         # Image subdirectory
└── private/            # Private videos (mixed with images)
    ├── video1/
    ├── video2/
    └── images/         # Image subdirectory
```

### New Structure
```
storage/
├── videos/
│   ├── public/         # Public videos only
│   │   ├── video1/
│   │   └── video2/
│   └── private/        # Private videos only
│       ├── video1/
│       └── video2/
└── images/
    ├── public/         # Public images only
    │   ├── logo.png
    │   └── banner.jpg
    └── private/        # Private images only
        └── secret.png
```

## Benefits

1. **Clear Separation** - Videos and images are now completely separated
2. **Consistency** - Same organizational pattern for both media types
3. **Scalability** - Easier to add more media types in the future
4. **Clarity** - Immediately obvious where each type of content lives

## Migration Process

### Automatic Migration (Recommended)

Use the provided migration script:

```bash
./migrate-video-storage.sh
```

This script will:
- ✅ Check for existing videos in old locations
- ✅ Create new directory structure
- ✅ Move videos to new locations
- ✅ Preserve all video files and directory structures
- ✅ Skip image directories (they stay in place)
- ✅ Optionally clean up old empty directories

### Manual Migration

If you prefer to migrate manually:

```bash
# 1. Create new directories
mkdir -p storage/videos/public
mkdir -p storage/videos/private

# 2. Move public videos (but not images)
cd storage/public
for dir in */; do
    if [ "$dir" != "images/" ]; then
        mv "$dir" ../videos/public/
    fi
done
cd ../..

# 3. Move private videos (but not images)
cd storage/private
for dir in */; do
    if [ "$dir" != "images/" ]; then
        mv "$dir" ../videos/private/
    fi
done
cd ../..

# 4. Verify
ls -la storage/videos/public/
ls -la storage/videos/private/
```

## Code Changes

The server code has been updated to use the new paths automatically:

**Before:**
```rust
let base_folder = if is_public { "public" } else { "private" };
```

**After:**
```rust
let base_folder = if is_public { "videos/public" } else { "videos/private" };
```

The server now creates the correct directory structure on startup:
```rust
std::fs::create_dir_all(storage_dir.join("videos/public"))?;
std::fs::create_dir_all(storage_dir.join("videos/private"))?;
std::fs::create_dir_all(storage_dir.join("images/public"))?;
std::fs::create_dir_all(storage_dir.join("images/private"))?;
```

## Verification

After migration, verify everything works:

### 1. Check Directory Structure
```bash
tree -L 3 storage/
# or
find storage -type d | sort
```

Expected output:
```
storage/
├── videos/
│   ├── public/
│   │   ├── welcome/
│   │   ├── webconjoint/
│   │   └── bbb/
│   └── private/
│       └── lesson1/
└── images/
    ├── public/
    └── private/
```

### 2. Test Server
```bash
# Start server
cargo run

# Test health endpoint
curl http://localhost:3000/health

# Test video list
curl http://localhost:3000/
```

### 3. Test Video Playback
Visit http://localhost:3000/ and click on any video link to verify playback works.

### 4. Check Database
Videos in the database don't need any changes - the slug-based lookup works the same:
```bash
sqlite3 video.db "SELECT slug, title FROM videos;"
```

## Troubleshooting

### Videos not found after migration

**Check file locations:**
```bash
ls -la storage/videos/public/
ls -la storage/videos/private/
```

**Check database entries:**
```bash
sqlite3 video.db "SELECT * FROM videos;"
```

**Verify slug matches directory name:**
The slug in the database should match the directory name in storage.

### Old directories still exist

The migration script preserves old directories if they contain the `images` subdirectory. This is intentional.

**To clean up manually:**
```bash
# Only do this after verifying videos work in new location
# and you've backed up images if needed

# Check what's left
ls -la storage/public/
ls -la storage/private/

# If only images directories remain, you can leave them
# or move them to storage/images/ if you prefer
```

### Server errors after migration

**Restart the server:**
```bash
# Stop
pkill -f video-server-rs

# Start fresh
cargo run
```

**Check logs:**
Look at the terminal output for any path-related errors.

**Verify permissions:**
```bash
chmod -R 755 storage/videos/
```

## Rollback

If you need to roll back to the old structure:

```bash
# 1. Stop the server
pkill -f video-server-rs

# 2. Move videos back
mv storage/videos/public/* storage/public/ 2>/dev/null
mv storage/videos/private/* storage/private/ 2>/dev/null

# 3. Revert code changes
git checkout src/main.rs

# 4. Restart server
cargo run
```

## For New Installations

If you're setting up a fresh installation, the new structure is created automatically:

```bash
# Just run the setup script
./setup-images.sh

# Or manually
mkdir -p storage/{videos,images}/{public,private}

# Then start the server
cargo run
```

## Database Schema

**No database changes required!** 

The videos table structure remains the same:
```sql
CREATE TABLE videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT 0
);
```

Video serving uses the slug to find the video directory:
- Slug `welcome` → `storage/videos/public/welcome/`
- Slug `lesson1` → `storage/videos/private/lesson1/`

## Migration Checklist

- [ ] Backup current storage directory: `cp -r storage storage.backup`
- [ ] Run migration script: `./migrate-video-storage.sh`
- [ ] Verify videos moved: `ls -la storage/videos/`
- [ ] Restart server: `cargo run`
- [ ] Test home page: http://localhost:3000/
- [ ] Test video playback
- [ ] Test private video access (login required)
- [ ] Verify images still accessible: http://localhost:3000/images
- [ ] Clean up old directories (optional)
- [ ] Remove backup after verification: `rm -rf storage.backup`

## Additional Resources

- **Migration Script**: `migrate-video-storage.sh`
- **Setup Script**: `setup-images.sh` (creates new structure)
- **Troubleshooting**: `TROUBLESHOOTING.md`
- **Complete Setup**: `SETUP_COMPLETE.md`

## Support

If you encounter issues:

1. Check the `TROUBLESHOOTING.md` guide
2. Verify directory structure matches expected layout
3. Check server logs for errors
4. Ensure database entries are correct
5. Test with a single video first before migrating all

## Summary

✅ **Videos moved to**: `storage/videos/{public,private}/`  
✅ **Images remain at**: `storage/images/{public,private}/`  
✅ **No database changes needed**  
✅ **Code automatically updated**  
✅ **Migration script provided**  
✅ **Backward compatible paths cleaned up**

The new structure provides better organization and sets the foundation for adding more media types in the future.
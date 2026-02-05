# Storage Structure Refactor - Public/Private Issue

**Issue Identified By**: User feedback  
**Date**: 2024-01-XX  
**Priority**: Medium-High (Design Issue)  
**Status**: 🔴 ISSUE IDENTIFIED - Solution Recommended

---

## 🐛 The Problem

### Current Storage Structure

```
storage/
├── videos/
│   ├── public/          ← Files with is_public = 1
│   │   ├── welcome/
│   │   │   ├── index.m3u8
│   │   │   └── segment-0001.ts
│   │   └── promo/
│   └── private/         ← Files with is_public = 0
│       ├── internal-meeting/
│       └── client-demo/
└── images/
    ├── public/          ← Files with is_public = 1
    │   ├── logo.png
    │   └── banner.jpg
    └── private/         ← Files with is_public = 0
        └── secret-docs.png
```

### The Issue: Database vs Filesystem Mismatch

**Scenario**: User toggles public/private setting

```sql
-- Step 1: Video uploaded as public
INSERT INTO videos (slug, filename, is_public) 
VALUES ('demo', 'demo.mp4', 1);

-- Stored at: storage/videos/public/demo/demo.mp4
-- ✅ Database and filesystem agree

-- Step 2: User changes to private
UPDATE videos SET is_public = 0 WHERE slug = 'demo';

-- Database says: is_public = 0 (private)
-- Filesystem says: File at storage/videos/public/demo/demo.mp4
-- ❌ MISMATCH!
```

**Current Code** (video-manager/src/lib.rs):
```rust
let base_folder = if is_public {
    "videos/public"
} else {
    "videos/private"
};
let full_path = state.storage_dir.join(base_folder).join(slug);
```

**Result**: 
- Access control checks `is_public` from database
- File lookup uses `is_public` to determine folder
- If these don't match → File not found!

---

## 🔥 Problems This Causes

### 1. State Synchronization Issue

```
Database State ≠ Filesystem State
   ↓
Requires manual file moves
   ↓
Race conditions, failures, complexity
```

### 2. File Must Be Moved

When toggling public/private:
```rust
// Pseudo-code needed:
if is_public changed {
    let old_path = if old_is_public { "public" } else { "private" };
    let new_path = if new_is_public { "public" } else { "private" };
    
    move_file(
        storage_dir.join(old_path).join(slug),
        storage_dir.join(new_path).join(slug)
    )?;
}
```

**Issues**:
- ❌ File move can fail (permissions, disk space)
- ❌ Race condition (access during move)
- ❌ Transaction issues (DB updated, file move fails)
- ❌ Rollback complexity
- ❌ Extra code complexity

### 3. Failed Move Scenarios

```
Scenario A: Move Fails After DB Update
  Database: is_public = 0 (private)
  File: storage/videos/public/demo/  (still in public folder)
  Result: File not accessible! (looks in private folder)

Scenario B: Partial Move (HLS videos with multiple files)
  Move index.m3u8 → Success
  Move segment-0001.ts → Fail
  Move segment-0002.ts → Not attempted
  Result: Broken video!

Scenario C: Concurrent Access
  User A: Watching video (reading from public/)
  User B: Toggles to private (moving files)
  Result: User A gets errors mid-playback
```

### 4. Backup/Restore Complexity

```
Backup:
  - Must maintain public/private folder structure
  - Restore must handle structure correctly
  - Schema migration must move files

Deployment:
  - New server must maintain structure
  - rsync must handle both folders
  - Docker volumes need both paths
```

---

## ✅ Recommended Solution: Single Folder Structure

### Proposed Structure

```
storage/
├── videos/
│   ├── welcome/
│   │   ├── index.m3u8
│   │   └── segment-0001.ts
│   ├── promo/
│   ├── internal-meeting/
│   └── client-demo/
└── images/
    ├── logo.png
    ├── banner.jpg
    └── secret-docs.png
```

**Key Change**: No public/private folders - just resource slugs!

### Why This Works

**Access control is NOT based on folder structure!**

```rust
// Access control checks database, not filesystem
let decision = access_control
    .check_access(context, Permission::Download)
    .await?;

if !decision.granted {
    return Err(StatusCode::UNAUTHORIZED);
}

// If access granted, serve from single location
let file_path = storage_dir
    .join("videos")
    .join(&slug)
    .join(&filename);
```

**Security is maintained by**:
1. ✅ Database field `is_public`
2. ✅ Access control middleware
3. ✅ Authentication checks
4. ✅ Group membership validation

**NOT** by folder structure!

---

## 📊 Benefits of Single Folder

| Benefit | Description |
|---------|-------------|
| **No File Moves** | Toggle is_public → only DB update |
| **Atomic Updates** | Single DB transaction, no file operations |
| **No Race Conditions** | File never moves, always at same path |
| **Simpler Code** | No folder logic, just: `videos/{slug}/` |
| **Easier Backups** | Single directory tree |
| **Faster Toggles** | UPDATE query only (~1ms vs ~100ms+ for file move) |
| **No Failures** | DB update can't fail due to filesystem |
| **Cleaner Paths** | Consistent structure |

---

## 🔄 Migration Plan

### Phase 1: Code Changes (Low Risk)

Update path resolution in handlers:

**Before** (video-manager/src/lib.rs):
```rust
let base_folder = if is_public {
    "videos/public"
} else {
    "videos/private"
};
let full_path = state
    .storage_dir
    .join(base_folder)
    .join(slug)
    .join(file_path);
```

**After**:
```rust
// Simple, no conditional
let full_path = state
    .storage_dir
    .join("videos")
    .join(slug)
    .join(file_path);
```

**Files to Update**:
- `crates/video-manager/src/lib.rs` - hls_proxy_handler (~line 487)
- `crates/image-manager/src/lib.rs` - upload_image_handler (~line 519)
- `crates/image-manager/src/lib.rs` - serve_image_handler (~line 1312)

### Phase 2: File Migration (One-Time)

```bash
#!/bin/bash
# migrate-storage.sh

# Migrate videos
if [ -d "storage/videos/public" ]; then
    echo "Migrating public videos..."
    mv storage/videos/public/* storage/videos/ 2>/dev/null || true
    rmdir storage/videos/public
fi

if [ -d "storage/videos/private" ]; then
    echo "Migrating private videos..."
    mv storage/videos/private/* storage/videos/ 2>/dev/null || true
    rmdir storage/videos/private
fi

# Migrate images
if [ -d "storage/images/public" ]; then
    echo "Migrating public images..."
    mv storage/images/public/* storage/images/ 2>/dev/null || true
    rmdir storage/images/public
fi

if [ -d "storage/images/private" ]; then
    echo "Migrating private images..."
    mv storage/images/private/* storage/images/ 2>/dev/null || true
    rmdir storage/images/private
fi

echo "Migration complete!"
```

### Phase 3: Update Main.rs Initialization

**Before**:
```rust
// Create video storage directories
std::fs::create_dir_all(storage_dir.join("videos/public"))?;
std::fs::create_dir_all(storage_dir.join("videos/private"))?;

// Create image storage directories
std::fs::create_dir_all(storage_dir.join("images/public"))?;
std::fs::create_dir_all(storage_dir.join("images/private"))?;
```

**After**:
```rust
// Create video storage directory
std::fs::create_dir_all(storage_dir.join("videos"))?;

// Create image storage directory
std::fs::create_dir_all(storage_dir.join("images"))?;
```

---

## 🧪 Testing Plan

### Test 1: Upload Works
```bash
# Upload public video
POST /api/videos/upload (is_public=1)
# Verify: File at storage/videos/{slug}/

# Upload private video
POST /api/videos/upload (is_public=0)
# Verify: File at storage/videos/{slug}/ (same structure!)
```

### Test 2: Access Control Still Works
```bash
# Public video - no auth
curl http://localhost:3000/watch/public-video
# Expected: ✅ Works (Layer 1: Public)

# Private video - no auth
curl http://localhost:3000/watch/private-video
# Expected: ❌ 401 Unauthorized

# Private video - with auth
curl -H "Authorization: Bearer <token>" http://localhost:3000/watch/private-video
# Expected: ✅ Works (Layer 4: Owner)
```

### Test 3: Toggle Public/Private
```sql
-- Make video public
UPDATE videos SET is_public = 1 WHERE slug = 'test-video';

-- Test access (no auth)
curl http://localhost:3000/watch/test-video
# Expected: ✅ Works immediately (no file move needed!)

-- Make video private
UPDATE videos SET is_public = 0 WHERE slug = 'test-video';

-- Test access (no auth)
curl http://localhost:3000/watch/test-video
# Expected: ❌ 401 Unauthorized (instant!)
```

### Test 4: Migration Script
```bash
# Setup test data
mkdir -p storage/videos/public/test1
mkdir -p storage/videos/private/test2
echo "test" > storage/videos/public/test1/video.mp4
echo "test" > storage/videos/private/test2/video.mp4

# Run migration
./migrate-storage.sh

# Verify
ls storage/videos/test1/video.mp4  # ✅ Should exist
ls storage/videos/test2/video.mp4  # ✅ Should exist
ls storage/videos/public           # ❌ Should not exist
ls storage/videos/private          # ❌ Should not exist
```

---

## 🔒 Security Verification

### Question: Is it secure without folder separation?

**Answer**: ✅ YES - Security is in the code, not the folder!

### Security Layers (Unchanged)

```
1. Access Control Middleware
   ↓
   Checks: is_public, user_id, group_id, access_code
   ↓
2. Permission Check
   ↓
   Verifies: Read, Download, Edit, Delete, Admin
   ↓
3. Audit Logging
   ↓
   Records: Who accessed what, when, how
   ↓
4. File Serving
   ↓
   Only happens AFTER all checks pass
```

### Filesystem Security

**Current** (with public/private folders):
```
storage/videos/public/   - chmod 755
storage/videos/private/  - chmod 700
```
❌ **This provides minimal security!**
- Web server can still read private/
- Access control must still check is_public
- Folder permissions are secondary defense

**Proposed** (single folder):
```
storage/videos/  - chmod 755
```
✅ **Primary security is access control code**
- Server checks is_public from database
- Access control validates permissions
- Folder permissions are OS-level only

**Conclusion**: Folder structure never provided real security. It was **security theater**.

---

## 📝 Code Changes Required

### File 1: video-manager/src/lib.rs

```rust
// Line ~487 in hls_proxy_handler
// BEFORE:
let base_folder = if is_public {
    "videos/public"
} else {
    "videos/private"
};
let full_path = state.storage_dir.join(base_folder).join(slug).join(file_path);

// AFTER:
let full_path = state.storage_dir.join("videos").join(slug).join(file_path);
```

### File 2: image-manager/src/lib.rs (Line ~519)

```rust
// In upload_image_handler
// BEFORE:
let base_folder = if is_public == 1 {
    "images/public"
} else {
    "images/private"
};
let target_path = storage_dir.join(base_folder).join(&final_filename);

// AFTER:
let target_path = storage_dir.join("images").join(&final_filename);
```

### File 3: image-manager/src/lib.rs (Line ~1312)

```rust
// In serve_image_handler
// BEFORE:
let base_folder = if is_public {
    "images/public"
} else {
    "images/private"
};
let full_path = state.storage_dir.join(base_folder).join(&filename);

// AFTER:
let full_path = state.storage_dir.join("images").join(&filename);
```

### File 4: main.rs

```rust
// In main() function
// BEFORE:
std::fs::create_dir_all(storage_dir.join("videos/public"))?;
std::fs::create_dir_all(storage_dir.join("videos/private"))?;
std::fs::create_dir_all(storage_dir.join("images/public"))?;
std::fs::create_dir_all(storage_dir.join("images/private"))?;

// AFTER:
std::fs::create_dir_all(storage_dir.join("videos"))?;
std::fs::create_dir_all(storage_dir.join("images"))?;
```

---

## ⚡ Implementation Steps

### Step 1: Code Changes (30 minutes)
- [ ] Update video-manager/src/lib.rs
- [ ] Update image-manager/src/lib.rs (2 locations)
- [ ] Update main.rs
- [ ] Test compilation: `cargo build`

### Step 2: Create Migration Script (15 minutes)
- [ ] Create `migrate-storage.sh`
- [ ] Test on development data
- [ ] Document rollback procedure

### Step 3: Test Thoroughly (1 hour)
- [ ] Test public video access
- [ ] Test private video access
- [ ] Test image serving
- [ ] Test toggle public/private
- [ ] Test file uploads
- [ ] Test access control still works

### Step 4: Deploy (15 minutes)
- [ ] Backup current storage/
- [ ] Run migration script
- [ ] Deploy new code
- [ ] Verify all resources accessible
- [ ] Monitor logs for errors

### Total Time: ~2 hours

---

## 🎯 Rollback Plan

If issues occur:

```bash
# 1. Stop server
sudo systemctl stop video-server

# 2. Restore code
git checkout <previous-commit>
cargo build --release

# 3. Restore files
rm -rf storage/videos/* storage/images/*
cp -r storage-backup/* storage/

# 4. Restart
sudo systemctl start video-server
```

**Or** keep old structure temporarily:

```bash
# Quick fix: symlinks
ln -s ../videos storage/videos/public
ln -s ../videos storage/videos/private
ln -s ../images storage/images/public
ln -s ../images storage/images/private
```

---

## 💡 Alternative: Hybrid Approach (Not Recommended)

**Keep folders but make toggles work**:

```rust
// On is_public update, also move file
async fn update_video_visibility(id: i32, is_public: bool) -> Result<()> {
    let video = get_video(id).await?;
    
    // Update database
    sqlx::query("UPDATE videos SET is_public = ? WHERE id = ?")
        .bind(is_public)
        .execute(&pool)
        .await?;
    
    // Move file
    let old_path = if video.is_public { "public" } else { "private" };
    let new_path = if is_public { "public" } else { "private" };
    
    if old_path != new_path {
        move_video_files(
            &format!("videos/{}/{}", old_path, video.slug),
            &format!("videos/{}/{}", new_path, video.slug),
        ).await?;
    }
    
    Ok(())
}
```

**Why Not Recommended**:
- ❌ Still complex
- ❌ Still can fail
- ❌ Still has race conditions
- ❌ Adds more code
- ❌ Doesn't solve the root problem

---

## ✅ Recommendation

**Implement Single Folder Structure**

**Pros**:
- ✅ Simple
- ✅ Reliable
- ✅ Fast
- ✅ No file moves
- ✅ No race conditions
- ✅ Easier to maintain

**Cons**:
- ⚠️ Need one-time migration
- ⚠️ ~2 hours work
- ⚠️ Need to test thoroughly

**Decision**: The benefits far outweigh the one-time migration cost.

---

## 📚 Related Documentation

- [MASTER_PLAN.md](./MASTER_PLAN.md) - Overall architecture
- [ACCESS_CONTROL_PROGRESS.md](./ACCESS_CONTROL_PROGRESS.md) - Access control implementation
- [PERMISSION_MANAGEMENT_GUIDE.md](./PERMISSION_MANAGEMENT_GUIDE.md) - How permissions work

---

**Status**: Issue Documented - Implementation Recommended  
**Priority**: Medium-High (affects maintainability)  
**Effort**: ~2 hours (low risk, high benefit)  
**Decision**: Pending user/team approval
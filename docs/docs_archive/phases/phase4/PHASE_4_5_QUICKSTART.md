# Phase 4.5: Storage Optimization & UI Consolidation - Quick Start Guide

**Status:** 🎯 STARTING NOW  
**Priority:** HIGH  
**Duration:** 3-4 weeks

---

## 🎯 What Are We Doing?

Two major optimizations:

1. **User-Based Storage** - Move from flat directories to user-based organization
2. **Unified Media UI** - Single interface for all media types at `/media`

---

## 🚀 Quick Start

### Day 1: Get Oriented

```bash
# Read these documents in order:
1. This file (you are here)
2. TODO_PHASE_4_5_STORAGE_UI.md (detailed task list)
3. MASTER_PLAN.md (Phase 4.5 section, lines ~1248-1631)

# Understand current architecture:
- Groups: Single group per file (virtual, DB only)
- Tags: Multiple tags per file (virtual, many-to-many)
- Storage: Currently flat (storage/videos/, storage/images/, etc.)
```

### Day 2-3: Storage Code Updates

**Priority 1: Common Storage Utilities**

```bash
# Create or update: crates/common/src/storage.rs
```

Add these functions:

```rust
impl StorageManager {
    /// Get user's storage root directory
    pub fn user_storage_root(&self, user_id: &str) -> PathBuf {
        self.base_path.join("users").join(user_id)
    }
    
    /// Ensure user's storage directories exist
    pub async fn ensure_user_storage(&self, user_id: &str) -> Result<()> {
        let user_root = self.user_storage_root(user_id);
        
        tokio::fs::create_dir_all(user_root.join("videos")).await?;
        tokio::fs::create_dir_all(user_root.join("images")).await?;
        tokio::fs::create_dir_all(user_root.join("documents")).await?;
        tokio::fs::create_dir_all(user_root.join("thumbnails")).await?;
        
        Ok(())
    }
    
    /// Get path for a media file (new structure)
    pub fn user_media_path(&self, user_id: &str, media_type: &str, slug: &str) -> PathBuf {
        self.user_storage_root(user_id)
            .join(media_type)
            .join(slug)
    }
    
    /// Find file location (checks both old and new structure)
    pub fn find_file(&self, user_id: &str, media_type: &str, slug: &str, filename: &str) -> Option<PathBuf> {
        // Try new location first
        let new_path = self.user_media_path(user_id, media_type, slug).join(filename);
        if new_path.exists() {
            return Some(new_path);
        }
        
        // Fall back to old location
        let old_path = self.base_path.join(media_type).join(slug).join(filename);
        if old_path.exists() {
            return Some(old_path);
        }
        
        None
    }
}
```

**Priority 2: Update Document Manager**

```bash
# Edit: crates/document-manager/src/storage.rs
```

Update `document_path()`:

```rust
pub fn document_path(&self, user_id: &str, slug: &str) -> PathBuf {
    self.storage_manager.user_media_path(user_id, "documents", slug)
}
```

**Priority 3: Update Image Manager**

```bash
# Edit: crates/image-manager/src/lib.rs
```

Update upload handler:

```rust
pub async fn upload_image_handler(...) {
    // ... existing code ...
    
    // NEW: Use user-based path
    let file_path = state.storage_dir
        .join("users")
        .join(&user_id)
        .join("images")
        .join(&stored_filename);
    
    // Ensure directory exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    // ... rest of upload logic ...
}
```

**Priority 4: Update Video Manager**

Similar changes to video storage paths.

---

## 📦 Week 1 Goals

**By End of Week 1:**
- [ ] Storage utilities implemented
- [ ] All managers updated to support user directories
- [ ] New uploads go to `storage/users/{user_id}/...`
- [ ] Old file retrieval still works (backward compatibility)
- [ ] Basic tests passing

**Commands to run:**

```bash
# Test that code compiles
cargo build

# Run tests
cargo test

# Test upload manually
curl -X POST http://localhost:8080/api/documents/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@test.pdf" \
  -F "title=Test Document"

# Verify file location
ls -la storage/users/*/documents/
```

---

## 🔄 Week 2: Migration Script

**Priority: Create Migration Script**

```bash
# Create: scripts/migrate_to_user_storage.rs
```

Script should:
1. Connect to database
2. Query all media files with user_id
3. For each file:
   - Create user directory
   - Move file from old to new location
   - Verify move (checksum)
   - Log progress
4. Generate report

**Test Migration:**

```bash
# 1. Backup current storage
cp -r storage storage-backup-$(date +%Y%m%d)

# 2. Run in dry-run mode
cargo run --bin migrate_to_user_storage -- --dry-run

# 3. Review output, then run for real
cargo run --bin migrate_to_user_storage

# 4. Verify results
./scripts/verify_migration.sh
```

---

## 🎨 Week 3: UI Enhancement

**Priority 1: Enhanced Media Hub**

```bash
# Edit: crates/media-hub/templates/media_list_tailwind.html
```

Add filter bar:

```html
<!-- Type Filters -->
<div class="flex gap-2">
    <a href="/media" class="btn btn-sm {% if !type_filter %}btn-primary{% endif %}">
        All ({{ total_count }})
    </a>
    <a href="/media?type=video" class="btn btn-sm {% if type_filter == 'video' %}btn-primary{% endif %}">
        🎥 Videos ({{ video_count }})
    </a>
    <a href="/media?type=image" class="btn btn-sm {% if type_filter == 'image' %}btn-primary{% endif %}">
        🖼️ Images ({{ image_count }})
    </a>
    <a href="/media?type=document" class="btn btn-sm {% if type_filter == 'document' %}btn-primary{% endif %}">
        📄 Documents ({{ document_count }})
    </a>
</div>

<!-- Tag Filters -->
<div class="flex gap-2 flex-wrap">
    {% for tag in available_tags %}
    <a href="/media?tags={{ tag.slug }}" class="badge badge-lg">
        {{ tag.name }} ({{ tag.count }})
    </a>
    {% endfor %}
</div>

<!-- Group Filter -->
<select name="group_id" class="select select-bordered">
    <option value="">All Groups</option>
    {% for group in user_groups %}
    <option value="{{ group.id }}">{{ group.name }} ({{ group.file_count }})</option>
    {% endfor %}
</select>
```

**Priority 2: Update Backend**

```bash
# Edit: crates/media-hub/src/routes.rs
```

Update `list_media_html()`:

```rust
async fn list_media_html(
    Query(params): Query<MediaListParams>,
    ...
) -> Result<Html<String>> {
    // Apply filters
    let mut query = "SELECT * FROM ...".to_string();
    
    if let Some(type_filter) = params.type_filter {
        query.push_str(" AND media_type = ?");
    }
    
    if let Some(group_id) = params.group_id {
        query.push_str(" AND group_id = ?");
    }
    
    if !params.tags.is_empty() {
        // JOIN with tags for filtering
        query.push_str(" JOIN ... ON ...");
    }
    
    // Execute and render
    ...
}
```

---

## 📊 Week 4: Polish & Release

**Checklist:**

- [ ] All tests passing
- [ ] Documentation updated
- [ ] Migration script tested on staging
- [ ] UI tested on mobile
- [ ] Code review complete
- [ ] Performance verified
- [ ] Rollback plan ready

**Release Steps:**

```bash
# 1. Tag release
git tag v0.5.0-storage-ui-optimization
git push --tags

# 2. Deploy to staging
./deploy-staging.sh

# 3. Run migration on staging
ssh staging "cd /app && ./scripts/migrate_to_user_storage.sh"

# 4. Verify staging works
curl https://staging.example.com/media

# 5. Deploy to production (low-traffic window)
./deploy-production.sh
```

---

## 🔍 Testing Checklist

### Storage Testing

```bash
# Upload as different users
curl -X POST .../upload -u user1:pass -F "file=@test.mp4"
curl -X POST .../upload -u user2:pass -F "file=@test.jpg"

# Verify file locations
ls storage/users/user1/videos/
ls storage/users/user2/images/

# Test retrieval
curl https://example.com/api/videos/test-slug

# Test old files still work (before migration)
curl https://example.com/api/videos/old-video
```

### UI Testing

```bash
# Test filters
open http://localhost:8080/media
open http://localhost:8080/media?type=video
open http://localhost:8080/media?type=image&tags=rust,tutorial
open http://localhost:8080/media?group_id=42

# Test upload
open http://localhost:8080/media/upload

# Test mobile
# Use browser dev tools, responsive mode
# Test on actual devices (iOS, Android)
```

---

## 📚 Key Documents

| Document | Purpose |
|----------|---------|
| `TODO_PHASE_4_5_STORAGE_UI.md` | Detailed task breakdown |
| `MASTER_PLAN.md` (Phase 4.5) | Architecture and design |
| `STORAGE_MIGRATION_GUIDE.md` | Migration procedures (to be created) |
| This file | Quick reference |

---

## 🆘 Common Issues

### Issue: "File not found after upload"

**Cause:** Path mismatch between upload and retrieval  
**Fix:** Check `find_file()` implementation

### Issue: "Migration script fails"

**Cause:** File permissions or missing user_id  
**Fix:** Run with sudo, check database for NULL user_ids

### Issue: "UI filters not working"

**Cause:** Query parameter parsing  
**Fix:** Check `MediaListParams` struct, verify query building

### Issue: "Performance slow with many files"

**Cause:** Missing indexes  
**Fix:** Run index creation SQL, verify with EXPLAIN QUERY PLAN

---

## 💡 Tips

1. **Start Small:** Test with a few files before full migration
2. **Keep Backups:** Always backup before migration
3. **Use Feature Flags:** Enable new structure gradually
4. **Monitor Logs:** Watch for path errors during transition
5. **Test Thoroughly:** Don't skip mobile testing
6. **Document Changes:** Update docs as you go

---

## 🎯 Success Metrics

**You know you're done when:**

- ✅ New uploads go to `storage/users/{user_id}/`
- ✅ Old files still accessible during transition
- ✅ Migration script completes successfully
- ✅ All files migrated to new structure
- ✅ `/media` page works with all filters
- ✅ Upload form works for all media types
- ✅ Mobile UI responsive and usable
- ✅ All tests passing
- ✅ Production deployment successful
- ✅ No user complaints

---

## 🤝 Getting Help

**Blocked?** Check:
1. This guide
2. TODO file for detailed steps
3. MASTER_PLAN for architecture context
4. Existing code (video-manager, image-manager)
5. Git history for similar changes

**Still stuck?** 
- Post in team chat
- Tag relevant team members
- Check related documentation

---

## 📅 Timeline Summary

| Week | Focus | Key Deliverable |
|------|-------|-----------------|
| 1 | Storage Code | New uploads use user directories |
| 2 | Migration | All files moved to new structure |
| 3 | UI Enhancement | Unified media interface working |
| 4 | Polish & Release | Production deployment |

---

**Ready to start?** 

```bash
# 1. Read TODO file
cat TODO_PHASE_4_5_STORAGE_UI.md

# 2. Create feature branch
git checkout -b feature/phase-4.5-storage-ui

# 3. Start with storage utilities
code crates/common/src/storage.rs

# 4. Update this file as you learn
# Good luck! 🚀
```

---

**Last Updated:** 2024-02-10  
**Phase Status:** 🎯 Starting Now
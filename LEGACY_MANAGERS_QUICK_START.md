# Legacy Managers Deactivation - Quick Start Guide

## TL;DR

We can now deactivate `image-manager` and `document-manager` crates because the unified `media-manager` and `media-hub` system provides all their functionality. The `video-manager` should be kept for complex HLS streaming.

## Why Deactivate?

✅ **Unified System Ready**
- `media_items` table consolidates all media types
- `media-hub` provides "All Media" view with filtering
- `media-manager` handles upload, serving, and markdown editing
- All features have parity or better

✅ **Benefits**
- Less code duplication
- Consistent user experience
- Single source of truth
- Easier maintenance
- Better architecture

⚠️ **Keep Video Manager**
- Complex HLS streaming logic
- Live streaming support
- Video transcoding pipeline
- Will migrate later

## Quick Comparison

| Feature | Legacy | Unified System | Status |
|---------|--------|----------------|--------|
| Image Gallery | `/images` | `/media?type=image` | ✅ Ready |
| Document List | `/documents` | `/media?type=document` | ✅ Ready |
| Image View | `/images/view/:slug` | `/images/:slug` | ✅ Ready |
| Document View | `/documents/:slug` | `/media/:slug/view` | ✅ Ready |
| Upload | Separate endpoints | `/api/media/upload` | ✅ Ready |
| Image Serving | image-manager | media-manager | ✅ Ready |
| Markdown Editor | Basic | Monaco + Preview | ✅ Better |
| Search | Separate | Unified | ✅ Better |
| Thumbnails | Per-type | Unified | ✅ Ready |

## How to Deactivate (Simple Method)

### Option 1: Use the Script (Recommended)

```bash
./deactivate_legacy_managers.sh
```

This will:
1. Comment out dependencies in `Cargo.toml`
2. Comment out imports in `src/main.rs`
3. Create redirect routes for backward compatibility
4. Update navbar links to unified routes

Then follow the on-screen instructions to complete the process.

### Option 2: Manual Steps

#### Step 1: Comment Out in Cargo.toml

```toml
[dependencies]
# DEACTIVATED - use media-manager instead
# image-manager = { path = "crates/image-manager" }
# document-manager = { path = "crates/document-manager" }

# Keep these
video-manager = { path = "crates/video-manager" }
media-manager = { path = "crates/media-manager" }
media-hub = { path = "crates/media-hub" }
```

#### Step 2: Comment Out in src/main.rs

```rust
// DEACTIVATED - using unified media system
// use image_manager::{image_routes, ImageManagerState};
// use document_manager::routes::{document_routes, DocumentManagerState};

// ... later in main() ...

// DEACTIVATED
// let image_state = Arc::new(ImageManagerState::new(pool.clone(), storage_dir.clone()));
// let document_state = Arc::new(DocumentManagerState::new(...));

// ... in router setup ...

// DEACTIVATED
// .merge(image_routes().with_state(image_state))
// .merge(document_routes().with_state(document_state))
```

#### Step 3: Add Redirects (Important!)

Create `src/legacy_redirects.rs`:

```rust
//! Legacy route redirects for backward compatibility

use axum::{extract::Path, response::Redirect, Router, routing::get};

pub fn legacy_redirect_routes() -> Router {
    Router::new()
        .route("/images", get(|| async { 
            Redirect::permanent("/media?type=image") 
        }))
        .route("/images/view/:slug", get(|Path(slug): Path<String>| async move {
            Redirect::permanent(&format!("/images/{}", slug))
        }))
        .route("/documents", get(|| async { 
            Redirect::permanent("/media?type=document") 
        }))
        .route("/documents/:slug", get(|Path(slug): Path<String>| async move {
            Redirect::permanent(&format!("/media/{}", slug))
        }))
}
```

Add to `src/main.rs`:

```rust
mod legacy_redirects;

// ... in router setup (BEFORE other routes) ...
.merge(legacy_redirects::legacy_redirect_routes())
```

#### Step 4: Update Navigation Links

In `templates/components/navbar.html`:

```html
<!-- Change from: -->
<a href="/images">🖼️ Images</a>
<a href="/documents">📄 Documents</a>

<!-- To: -->
<a href="/media?type=image">🖼️ Images</a>
<a href="/media?type=document">📄 Documents</a>
```

#### Step 5: Test

```bash
# Build
cargo build

# Run
cargo run

# Test redirects
curl -I http://localhost:8080/images
# Should return: 308 Permanent Redirect
# Location: /media?type=image

curl -I http://localhost:8080/documents
# Should return: 308 Permanent Redirect  
# Location: /media?type=document
```

## Testing Checklist

After deactivation, verify:

- [ ] Upload images via `/media/upload` or "All Media" UI
- [ ] Upload documents via `/media/upload` or "All Media" UI
- [ ] View all images at `/media?type=image`
- [ ] View all documents at `/media?type=document`
- [ ] Click individual images - they load correctly
- [ ] Click individual documents - markdown shows preview/raw toggle
- [ ] Edit markdown documents - Monaco editor works
- [ ] Save markdown documents - changes persist
- [ ] Delete media items - works for all types
- [ ] Search across all media types
- [ ] Filter by tags, public/private, etc.
- [ ] Old links redirect correctly (test `/images`, `/documents`)
- [ ] Thumbnails display for all media types
- [ ] Download works for documents

## Rollback Plan

If issues arise, rollback is simple:

```bash
# Restore backups
cp Cargo.toml.backup Cargo.toml
cp src/main.rs.backup src/main.rs

# Remove redirect module
rm src/legacy_redirects.rs

# Rebuild
cargo clean
cargo build
```

## What Stays, What Goes

### ✅ KEEP (Active System)

- `video-manager` - HLS streaming, transcoding, live support
- `media-manager` - Unified upload, serving, markdown editing
- `media-hub` - "All Media" UI, search, filtering
- `common` - Shared models, utilities
- `access-control` - Permissions system

### ⚠️ DEACTIVATE (Legacy)

- `image-manager` - Replaced by media-manager + media-hub
- `document-manager` - Replaced by media-manager + media-hub

### 📋 Archive (Later)

After 1-2 weeks of monitoring with no issues:

```bash
mkdir -p archive/legacy-managers
mv crates/image-manager archive/legacy-managers/
mv crates/document-manager archive/legacy-managers/
```

Then remove from Cargo.toml completely.

## FAQ

**Q: Will old bookmarks break?**  
A: No, redirects ensure old URLs work.

**Q: Will API clients break?**  
A: Keep `/api/images` and `/api/documents` as aliases if needed.

**Q: What about non-markdown documents?**  
A: Currently redirects to media detail. Add viewers if needed.

**Q: Can we deactivate video-manager too?**  
A: Not yet - HLS streaming is complex and needs careful migration.

**Q: How long to monitor before full removal?**  
A: Recommended 1-2 weeks minimum, 1 month to be safe.

**Q: What if we discover missing features?**  
A: Add them to media-manager, then resume deactivation.

## Timeline Recommendation

- **Day 1**: Deactivate and add redirects
- **Week 1**: Monitor logs, fix any issues
- **Week 2-4**: Continue monitoring, gather feedback
- **After 1 month**: Archive legacy crates if no issues

## Key Files to Watch

After deactivation, monitor these for errors:

- `server.log` - Check for 404s or errors
- Browser console - Check for failed API calls
- User feedback - Note any confusion or issues

## Success Criteria

Deactivation is successful when:

1. ✅ All media types work through unified system
2. ✅ No errors in logs related to media operations
3. ✅ Old URLs redirect correctly
4. ✅ No user complaints about missing features
5. ✅ Code is cleaner and more maintainable

## Documentation to Update

After successful deactivation:

- [ ] README.md - Remove image-manager and document-manager mentions
- [ ] ARCHITECTURE_DECISIONS.md - Add ADR for consolidation
- [ ] API documentation - Update endpoint references
- [ ] User guides - Update screenshots and instructions

## Support

For issues or questions:

1. Check `docs/LEGACY_MANAGERS_DEACTIVATION.md` for detailed plan
2. Review commit history for migration patterns
3. Test in dev environment before production
4. Keep backups until fully validated

---

**Status**: 🟢 Ready to Begin  
**Risk Level**: 🟡 Low-Medium (with redirects)  
**Effort**: ~1 day deactivation, 1-4 weeks monitoring  
**Impact**: High (cleaner codebase, better UX)

**Last Updated**: 2025-02-12
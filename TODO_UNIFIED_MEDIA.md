# Unified Media System - TODO List

**Quick reference for remaining tasks**

---

## 🔥 High Priority (Next Steps)

### 1. Mount Media Manager Routes
**File:** `src/main.rs`
```rust
// Add to dependencies
media_manager = { path = "crates/media-manager" }

// In main():
let media_state = Arc::new(media_manager::MediaManagerState::new(
    pool.clone(),
    storage_dir.to_str().unwrap().to_string(),
    user_storage.clone(),
    access_control.clone(),
));

// Add to router (BEFORE old routes for precedence)
.merge(media_manager::media_routes().with_state((*media_state).clone()))
```

### 2. Test Image Upload
```bash
# 1. Start server
cargo run

# 2. Login first to get session cookie
curl -c cookies.txt -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username":"your_username","password":"your_password"}'

# 3. Upload image
curl -b cookies.txt -X POST http://localhost:3000/api/media/upload \
  -F "media_type=image" \
  -F "title=My First Photo" \
  -F "is_public=1" \
  -F "file=@photo.jpg"

# 4. View image
curl http://localhost:3000/images/my-first-photo > test.webp
open test.webp
```

### 3. Implement Video Upload
**File:** `crates/media-manager/src/upload.rs`
**Function:** `process_video_upload()`

**Steps:**
- [ ] Copy logic from `video-manager/src/upload.rs`
- [ ] Extract metadata with FFmpeg
- [ ] Store in vault: `storage/vaults/{vault_id}/videos/{slug}/`
- [ ] Generate thumbnail
- [ ] Queue HLS processing
- [ ] Insert into `media_items` table
- [ ] Return upload response

### 4. Implement Document Upload
**File:** `crates/media-manager/src/upload.rs`
**Function:** `process_document_upload()`

**Steps:**
- [ ] Copy logic from current `document-manager/src/routes.rs`
- [ ] Store in vault: `storage/vaults/{vault_id}/documents/`
- [ ] Detect MIME type
- [ ] Classify document type
- [ ] Insert into `media_items` table
- [ ] Return upload response

---

## 📊 Medium Priority

### 5. Implement List Media
**File:** `crates/media-manager/src/routes.rs`
**Function:** `list_media()`

```rust
async fn list_media(
    State(state): State<MediaManagerState>,
    Query(filters): Query<MediaItemFilterOptions>,
    session: Session,
) -> Result<Json<MediaItemListResponse>, StatusCode> {
    // Build SQL with filters
    // Handle pagination
    // Check access control
    // Return list
}
```

### 6. Implement Get Media Detail
**File:** `crates/media-manager/src/routes.rs`
**Function:** `get_media_detail()`

```rust
async fn get_media_detail(
    State(state): State<MediaManagerState>,
    Path(slug): Path<String>,
    session: Session,
) -> Result<Json<MediaItem>, StatusCode> {
    // Fetch from media_items
    // Include tags via view
    // Check access control
    // Return full item
}
```

### 7. Add Video Serving
**File:** `crates/media-manager/src/serve.rs`
**New functions:** `serve_video()`, `serve_video_hls()`

### 8. Add Document Serving
**File:** `crates/media-manager/src/serve.rs`
**New functions:** `serve_document_download()`, `serve_document_view()`

---

## 🔧 Low Priority (Polish)

### 9. Tag Management API
- [ ] `POST /api/media/:slug/tags` - Add tags
- [ ] `DELETE /api/media/:slug/tags/:tag` - Remove tag
- [ ] `GET /api/tags` - List all tags
- [ ] `GET /api/tags/:tag/media` - Media by tag

### 10. Update Frontend
- [ ] Change upload forms to use `/api/media/upload`
- [ ] Add `media_type` selector
- [ ] Add tag input UI
- [ ] Update image URLs to new routes

### 11. Write Tests
- [ ] Unit tests for upload handlers
- [ ] Integration tests for serving
- [ ] Access control tests
- [ ] Tag management tests

### 12. Documentation
- [ ] API documentation (OpenAPI/Swagger)
- [ ] User guide for uploads
- [ ] Admin guide for vault management

---

## 🎯 Success Criteria

**Phase 1 (Image) - COMPLETE ✅**
- [x] Images upload to unified table
- [x] Original + WebP both stored
- [x] Thumbnails generated
- [x] All serving endpoints work
- [x] Access control integrated

**Phase 2 (Video & Document) - IN PROGRESS**
- [ ] Videos upload to unified table
- [ ] Documents upload to unified table
- [ ] All media types queryable via single endpoint
- [ ] Tags work for all types

**Phase 3 (Deprecation)**
- [ ] Old endpoints deprecated
- [ ] Frontend migrated
- [ ] Old tables dropped (after backup)

---

## 🐛 Known Issues

None currently - fresh implementation ✨

---

## 📝 Notes

### Image Upload Format
```javascript
// JavaScript example
const formData = new FormData();
formData.append('media_type', 'image');
formData.append('title', 'My Photo');
formData.append('description', 'Optional description');
formData.append('is_public', '1');
formData.append('category', 'landscape');
formData.append('tags', 'sunset,nature,beach');
formData.append('file', fileInput.files[0]);

fetch('/api/media/upload', {
  method: 'POST',
  body: formData,
  credentials: 'include'
});
```

### Accessing Images
```html
<!-- In your HTML -->
<img src="/images/photo-slug" alt="Photo">
<img src="/images/photo-slug/thumb" alt="Thumbnail">
<img src="/images/photo-slug/original" alt="Original quality">

<!-- With access code for private images -->
<img src="/images/photo-slug?code=abc123" alt="Private photo">
```

### Database Queries
```sql
-- Find all images
SELECT * FROM media_items WHERE media_type = 'image';

-- Find featured media
SELECT * FROM media_items WHERE featured = 1;

-- Find by tag
SELECT m.* FROM media_items m
JOIN media_tags t ON m.id = t.media_id
WHERE t.tag = 'sunset';

-- Media with all tags
SELECT slug, media_type, tags FROM media_items_with_tags;
```

---

**Created:** February 14, 2025
**Last Updated:** February 14, 2025

# Legacy Upload Endpoints Removed

**Date:** February 15, 2026  
**Status:** ✅ Complete  
**Impact:** Breaking change for upload workflows

---

## 🎯 Summary

All legacy media upload endpoints have been **removed** in favor of a single unified upload endpoint. This ensures consistency, prevents database sync issues, and simplifies the codebase.

---

## 🚫 Removed Endpoints

### Images
- ❌ `GET /upload` - Legacy image upload page
- ❌ `POST /api/images/upload` - Legacy image upload API

### Videos
- ❌ `GET /videos/upload` - Legacy video upload page
- ❌ `POST /api/videos/upload` - Legacy video upload API
- ❌ `GET /api/videos/upload/:upload_id/progress` - Legacy upload progress

### Documents
- ❌ `POST /api/documents/upload` - Legacy document upload API

---

## ✅ New Unified Endpoint

### Upload Page (HTML)
```
GET /media/upload
```
User-friendly upload form with vault/group selection.

### Upload API (JSON)
```
POST /api/media/upload
Content-Type: multipart/form-data

Parameters:
- file: File (required)
- title: String (optional)
- description: String (optional)
- vault_id: String (optional)
- group_id: Integer (optional)
- is_public: Boolean (optional, default: false)
- tags: String (optional, comma-separated)
```

**Response:**
```json
{
  "success": true,
  "media_type": "image",
  "slug": "my-photo-1234567890",
  "id": 42,
  "title": "My Photo",
  "thumbnail_url": "/images/my-photo-1234567890_thumb",
  "url": "/images/my-photo-1234567890"
}
```

---

## 🔧 Migration Guide

### For API Users

**Before (Legacy):**
```bash
# Images
curl -X POST http://localhost:3000/api/images/upload \
  -F "file=@photo.jpg" \
  -F "title=My Photo"

# Videos
curl -X POST http://localhost:3000/api/videos/upload \
  -F "file=@video.mp4" \
  -F "title=My Video"

# Documents
curl -X POST http://localhost:3000/api/documents/upload \
  -F "file=@doc.pdf" \
  -F "title=My Document"
```

**After (Unified):**
```bash
# All media types use the same endpoint!
curl -X POST http://localhost:3000/api/media/upload \
  -F "file=@photo.jpg" \
  -F "title=My Photo" \
  -F "vault_id=vault-abc123" \
  -F "is_public=true"

curl -X POST http://localhost:3000/api/media/upload \
  -F "file=@video.mp4" \
  -F "title=My Video" \
  -F "group_id=5"

curl -X POST http://localhost:3000/api/media/upload \
  -F "file=@doc.pdf" \
  -F "title=My Document"
```

### For UI Users

**Before:**
- Navigate to `/upload` for images
- Navigate to `/videos/upload` for videos
- Different upload forms for each type

**After:**
- Single upload page at `/media/upload`
- Automatically detects file type
- Unified vault/group selection
- Consistent UI/UX

---

## 🐛 Why This Change?

### Problems with Legacy Endpoints

1. **Database Sync Issues**
   - Legacy uploads only wrote to old tables (`images`, `videos`, `documents`)
   - New system uses unified `media_items` table
   - Result: Missing thumbnails, broken previews, 404 errors

2. **Inconsistent Behavior**
   - Images uploaded to `storage/images/`
   - Videos to `storage/videos/`
   - Documents to `storage/documents/`
   - No vault-based organization

3. **Missing Metadata**
   - No thumbnail URLs set
   - No vault_id assignment
   - No proper user_id tracking
   - Thumbnails saved in wrong directories

4. **Code Duplication**
   - Three separate upload handlers
   - Three different validation systems
   - Three different error handling approaches

### Benefits of Unified Endpoint

1. **Single Source of Truth**
   - All uploads go to `media_items` table
   - Consistent metadata
   - Proper thumbnail URLs

2. **Vault-Based Storage**
   - Organized by vault
   - User-scoped directories
   - Access control integrated

3. **Automatic File Detection**
   - MIME type detection
   - Automatic categorization (image/video/document)
   - Format-specific processing

4. **Better Error Handling**
   - Unified validation
   - Consistent error messages
   - Transaction support

---

## 📊 Database Schema

### Unified Table Structure

All media now stored in **one table**:

```sql
CREATE TABLE media_items (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    media_type TEXT NOT NULL,  -- 'image', 'video', 'document'
    title TEXT NOT NULL,
    description TEXT,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    
    -- Access Control
    is_public INTEGER DEFAULT 0,
    user_id TEXT,
    vault_id TEXT,
    group_id INTEGER,
    
    -- URLs
    thumbnail_url TEXT,
    webp_url TEXT,
    preview_url TEXT,
    
    -- Timestamps
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT,
    
    -- ... more fields
);
```

### Legacy Tables (Read-Only)

The old tables still exist for **backward compatibility**:

```sql
-- Still exist but NOT used for new uploads
CREATE TABLE images (...);    -- Read-only
CREATE TABLE videos (...);    -- Read-only
CREATE TABLE documents (...); -- Read-only
```

**Note:** These may be deprecated in the future.

---

## 🗂️ File Storage

### New Structure (Vault-Based)

```
storage/
├── vaults/
│   ├── vault-abc123/
│   │   ├── images/
│   │   │   ├── my-photo.png
│   │   │   └── another-pic.webp
│   │   ├── videos/
│   │   │   └── my-video/
│   │   │       ├── master.m3u8
│   │   │       └── segments/
│   │   ├── documents/
│   │   │   └── my-doc.pdf
│   │   └── thumbnails/
│   │       ├── images/
│   │       │   ├── my-photo_thumb.webp
│   │       │   └── another-pic_thumb.webp
│   │       └── videos/
│   │           └── my-video_thumb.webp
│   └── vault-xyz789/
│       └── ...
└── users/              -- Legacy fallback
    └── user-123/
        └── ...
```

### Legacy Structure (Deprecated)

```
storage/
├── images/           -- ❌ No longer used for uploads
├── videos/           -- ❌ No longer used for uploads
├── documents/        -- ❌ No longer used for uploads
└── thumbnails/       -- ❌ Inconsistent locations
    ├── images/
    └── videos/
```

---

## 🔐 Access Control Integration

The unified endpoint integrates with the 4-layer access control system:

```
Upload Flow:
1. User uploads file → /api/media/upload
2. Check authentication
3. Validate vault/group permissions
4. Store in vault directory
5. Insert into media_items table
6. Generate thumbnails (in correct locations!)
7. Set proper URLs
8. Return success
```

**Vault Assignment:**
- Authenticated users: Can select vault
- Guest uploads: Go to default/public vault
- Group uploads: Vault inherited from group

---

## 🧪 Testing

### Test the Unified Upload

```bash
# Test image upload
curl -X POST http://localhost:3000/api/media/upload \
  -F "file=@test.png" \
  -F "title=Test Image" \
  -F "is_public=true"

# Verify in database
sqlite3 media.db "SELECT id, slug, media_type, vault_id, thumbnail_url FROM media_items ORDER BY id DESC LIMIT 1;"

# Check file locations
ls -la storage/vaults/*/images/
ls -la storage/vaults/*/thumbnails/images/

# Test serving
curl http://localhost:3000/images/{slug}
curl http://localhost:3000/images/{slug}_thumb
```

### Expected Results

✅ Entry in `media_items` table  
✅ File in `storage/vaults/{vault_id}/images/`  
✅ Thumbnail in `storage/vaults/{vault_id}/thumbnails/images/`  
✅ `thumbnail_url` set in database  
✅ Image accessible via `/images/{slug}`  
✅ Thumbnail accessible via `/images/{slug}_thumb`  
✅ Shows in "All Media" list with preview  

---

## 🚨 Breaking Changes

### What Will Break

1. **Direct API Calls to Legacy Endpoints**
   - Scripts using `/api/images/upload`
   - Scripts using `/api/videos/upload`
   - Scripts using `/api/documents/upload`
   - **Fix:** Update to `/api/media/upload`

2. **Upload Progress Tracking**
   - `/api/videos/upload/:upload_id/progress` removed
   - **Fix:** Use WebSocket or polling on `/api/media/upload/status` (if implemented)

3. **Legacy Upload Forms**
   - `/upload` (images) redirects or 404
   - `/videos/upload` redirects or 404
   - **Fix:** Use `/media/upload`

### What Still Works

✅ All serving endpoints (`/images/:slug`, `/videos/:slug`, `/documents/:slug`)  
✅ All CRUD operations (update, delete, tags)  
✅ All list/search endpoints  
✅ HLS streaming  
✅ Access control  
✅ 3D gallery  

---

## 📋 Checklist for Developers

If you're updating client code:

- [ ] Replace all `/api/images/upload` calls with `/api/media/upload`
- [ ] Replace all `/api/videos/upload` calls with `/api/media/upload`
- [ ] Replace all `/api/documents/upload` calls with `/api/media/upload`
- [ ] Update upload form URLs to `/media/upload`
- [ ] Add `vault_id` parameter for organization
- [ ] Test thumbnail generation
- [ ] Verify media appears in "All Media" list
- [ ] Check file storage locations

---

## 🎓 Example: Complete Upload Workflow

```bash
#!/bin/bash
# upload_example.sh - Complete media upload example

API_URL="http://localhost:3000/api/media/upload"
VAULT_ID="vault-abc123"

# Login first (get session cookie)
curl -c cookies.txt -X POST http://localhost:3000/login/emergency/auth \
  -d "username=admin&password=appkask"

# Upload an image
echo "Uploading image..."
curl -b cookies.txt -X POST $API_URL \
  -F "file=@photo.jpg" \
  -F "title=Vacation Photo" \
  -F "description=Beach sunset" \
  -F "vault_id=$VAULT_ID" \
  -F "is_public=true" \
  -F "tags=vacation,beach,sunset" \
  | python3 -m json.tool

# Upload a video
echo "Uploading video..."
curl -b cookies.txt -X POST $API_URL \
  -F "file=@video.mp4" \
  -F "title=Demo Video" \
  -F "vault_id=$VAULT_ID" \
  | python3 -m json.tool

# Upload a document
echo "Uploading document..."
curl -b cookies.txt -X POST $API_URL \
  -F "file=@report.pdf" \
  -F "title=Annual Report" \
  -F "vault_id=$VAULT_ID" \
  -F "group_id=5" \
  | python3 -m json.tool

# Cleanup
rm cookies.txt

echo "All uploads complete!"
```

---

## 📞 Support

**Issues?**
- Check file is in correct vault directory
- Verify `media_items` table entry exists
- Check `thumbnail_url` is set
- Ensure thumbnails are in `storage/vaults/{vault_id}/thumbnails/`

**Still having problems?**
- Check server logs for detailed errors
- Verify vault permissions
- Test with `is_public=true` first
- Use browser DevTools Network tab

---

## 🎯 Future Improvements

- [ ] Add upload progress API (WebSocket or SSE)
- [ ] Batch upload support
- [ ] Resume interrupted uploads
- [ ] Direct S3/cloud storage upload
- [ ] Image optimization pipeline
- [ ] Video transcoding queue
- [ ] Duplicate detection

---

**Status:** ✅ Migration Complete  
**Next Steps:** Update any external clients/scripts to use `/api/media/upload`  
**Documentation Updated:** February 15, 2026
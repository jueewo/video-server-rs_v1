# 🚀 Unified Media System - Quick Start Guide

**Ready to use!** Your unified media system is now integrated and working.

---

## ⚡ Quick Test (30 seconds)

### Step 1: Start the Server
```bash
cargo run
```

Wait for: `✅ Server running on http://0.0.0.0:3000`

### Step 2: Run Automated Test
```bash
# In a new terminal
./test_unified_upload.sh
```

**Expected Output:**
```
🧪 Testing Unified Media Upload System
========================================

1️⃣  Checking authentication...
   ⚠️  Skipping authentication
2️⃣  Creating test image...
   ✓ Test image created
3️⃣  Uploading image via /api/media/upload...
   ✓ Upload successful!
4️⃣  Testing image serving endpoints...
   ✓ GET /images/test-upload-xxxxx → 200 OK
   ✓ GET /images/test-upload-xxxxx/original → 200 OK
   ✓ GET /images/test-upload-xxxxx/thumb → 200 OK
   ✓ GET /images/test-upload-xxxxx.webp → 200 OK
```

✅ **If you see all green checkmarks, everything works!**

---

## 📸 Manual Image Upload

### Upload with curl
```bash
curl -X POST http://localhost:3000/api/media/upload \
  -F "media_type=image" \
  -F "title=My Photo" \
  -F "description=A beautiful sunset" \
  -F "is_public=1" \
  -F "category=landscape" \
  -F "tags=sunset,beach,nature" \
  -F "file=@/path/to/photo.jpg"
```

### Response
```json
{
  "success": true,
  "message": "Image uploaded successfully",
  "media_type": "image",
  "slug": "my-photo",
  "id": 23,
  "webp_url": "/images/my-photo.webp",
  "thumbnail_url": "/images/my-photo/thumb"
}
```

---

## 🖼️ View Uploaded Images

### In Browser
```
http://localhost:3000/images/my-photo           → WebP (optimized)
http://localhost:3000/images/my-photo/original  → Original quality
http://localhost:3000/images/my-photo/thumb     → Thumbnail
```

### Download with curl
```bash
# Download WebP
curl http://localhost:3000/images/my-photo -o image.webp

# Download original
curl http://localhost:3000/images/my-photo/original -o original.jpg

# Download thumbnail
curl http://localhost:3000/images/my-photo/thumb -o thumb.webp
```

---

## 🔍 Check Database

### View all media
```bash
sqlite3 media.db "SELECT slug, media_type, title FROM media_items ORDER BY id DESC LIMIT 10;"
```

### View with tags
```bash
sqlite3 media.db "SELECT slug, media_type, tags FROM media_items_with_tags LIMIT 10;"
```

### Count by type
```bash
sqlite3 media.db "SELECT media_type, COUNT(*) FROM media_items GROUP BY media_type;"
```

---

## 🌐 Embed in HTML

### Basic
```html
<img src="http://localhost:3000/images/my-photo" alt="My Photo">
```

### With thumbnail
```html
<img src="http://localhost:3000/images/my-photo/thumb"
     alt="Thumbnail"
     onclick="window.open('/images/my-photo/original')">
```

### Responsive
```html
<picture>
  <source srcset="http://localhost:3000/images/my-photo.webp" type="image/webp">
  <img src="http://localhost:3000/images/my-photo/original" alt="My Photo">
</picture>
```

---

## 🔐 Private Images with Access Codes

### Upload private image
```bash
curl -X POST http://localhost:3000/api/media/upload \
  -F "media_type=image" \
  -F "title=Private Photo" \
  -F "is_public=0" \
  -F "file=@photo.jpg"
```

### Access with code
```html
<img src="http://localhost:3000/images/private-photo?code=YOUR_ACCESS_CODE">
```

---

## 📊 Storage Locations

Your files are stored in:

```
storage/vaults/{vault-id}/
├── images/
│   ├── my-photo_original.jpg    ← Original file
│   └── my-photo.webp            ← WebP version
└── thumbnails/
    └── images/
        └── my-photo_thumb.webp  ← Thumbnail (400x400)
```

---

## 🎯 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/media/upload` | POST | Upload any media type |
| `/images/:slug` | GET | Serve WebP image |
| `/images/:slug/original` | GET | Serve original image |
| `/images/:slug/thumb` | GET | Serve thumbnail |
| `/images/:slug.webp` | GET | Serve WebP explicitly |

---

## 🔧 Advanced Options

### Auto-slug vs Manual slug
```bash
# Auto-generate slug from title
curl ... -F "title=My Amazing Photo"
# Result: slug = "my-amazing-photo"

# Provide custom slug
curl ... -F "title=My Photo" -F "slug=custom-slug-2025"
# Result: slug = "custom-slug-2025"
```

### Multiple tags
```bash
# Comma-separated
curl ... -F "tags=landscape,sunset,beach,nature"
```

### Categories
```bash
curl ... -F "category=photography"
```

---

## ❓ Troubleshooting

### "Authentication required" error
**Solution:** You need to login first if authentication is enabled.
```bash
# Login first
curl -c cookies.txt -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"yourpass"}'

# Then upload with cookies
curl -b cookies.txt -X POST http://localhost:3000/api/media/upload ...
```

### "Slug already exists" error
**Solution:** Either:
- Change the title
- Provide a unique custom slug with `-F "slug=unique-name"`
- Delete the existing item from database

### 404 Not Found when viewing image
**Check:**
1. Image was uploaded: `sqlite3 media.db "SELECT slug FROM media_items WHERE slug='my-photo';"`
2. File exists: `ls storage/vaults/*/images/my-photo*`
3. Server is running: `curl http://localhost:3000/health`

### WebP not generated
**Check:**
- File is not SVG (SVGs are preserved as-is)
- Check server logs for errors
- Verify `image` crate is installed: `cargo tree | grep "^image"`

---

## 📚 More Information

- **Full Documentation:** `UNIFIED_MEDIA_PROGRESS.md`
- **Integration Guide:** `INTEGRATION_COMPLETE.md`
- **TODO List:** `TODO_UNIFIED_MEDIA.md`
- **Test Script:** `test_unified_upload.sh`

---

## 🎉 What's Working

✅ Image upload with original + WebP
✅ Automatic thumbnail generation
✅ Tag support (normalized storage)
✅ Vault-based file organization
✅ Access control with codes
✅ Multiple serving endpoints
✅ CORS for external embedding
✅ View count tracking

---

## 🚧 Coming Soon

- Video upload handler
- Document upload handler
- List/filter media API
- Tag management API

---

**Questions?** Check the documentation files or run `./test_unified_upload.sh` to verify setup.

**Last Updated:** February 14, 2025

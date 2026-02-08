# CRUD Operations Clarification

**Purpose:** Clarify what CRUD operations do vs automatic processing  
**Last Updated:** February 2026  
**Status:** ✅ Aligned with actual implementation

---

## 🎯 Quick Answer

**CRUD operations are for METADATA management, NOT file transformation.**

✅ **CRUD does:**
- Update title, description, slug
- Add/remove tags
- Change visibility (public/private)
- Change group assignment
- Delete resources

❌ **CRUD does NOT:**
- Resize/rescale images
- Transcode videos
- Change file formats
- Edit file content

---

## 📋 What Happens During Upload vs Update

### During Upload (Automatic Processing)

**Images:**
```
1. User uploads: photo.jpg (5MB, 4000x3000)
   ↓
2. Server automatically:
   ✅ Generates thumbnail (200x150)
   ✅ Converts to WebP (optimization)
   ✅ Extracts EXIF data
   ✅ Stores original + processed versions
   ↓
3. Saved to database with metadata
```

**Videos:**
```
1. User uploads: video.mp4 (100MB)
   ↓
2. Server automatically:
   ✅ Generates thumbnail/poster
   ✅ Extracts duration, resolution, codec
   ✅ Creates HLS streams (if configured)
   ✅ Stores metadata
   ↓
3. Saved to database with metadata
```

**Key Point:** File processing happens ONCE at upload time, automatically.

---

### During Update (Metadata Only)

**What you CAN update:**

```json
PUT /api/videos/my-video
{
  "title": "New Title",
  "description": "Updated description",
  "slug": "new-slug",
  "visibility": "public",
  "group_id": 42
}
```

**What happens:**
- ✅ Database record updated
- ✅ Metadata changed
- ❌ NO file re-processing
- ❌ NO file modification
- ❌ NO re-encoding

**Result:** Only metadata changes, files stay the same.

---

## ✅ Standard CRUD Operations

### Create (Upload)

**Endpoint:** `POST /api/videos` or `POST /api/images`

**What it does:**
1. Accept file upload
2. **Automatically process file** (thumbnails, WebP conversion, etc.)
3. Store file(s) on disk
4. Save metadata to database
5. Return resource details

**Files created:**
```
Images:
├── original.jpg (stored)
├── original.webp (auto-converted)
└── thumbnail.webp (auto-generated)

Videos:
├── original.mp4 (stored)
├── thumbnail.jpg (auto-generated)
└── poster.jpg (auto-generated)
```

### Read

**Endpoints:**
- `GET /api/videos` - List all
- `GET /api/videos/:slug` - Get one
- `GET /api/videos/by-tag/:tag` - Filter by tag

**What it does:**
- Query database
- Return metadata (title, description, slug, tags, etc.)
- Return URLs to files

**No file processing involved.**

### Update

**Endpoint:** `PUT /api/videos/:slug`

**What you can update:**
```json
{
  "title": "string",           // ✅ Yes
  "description": "string",      // ✅ Yes
  "slug": "string",            // ✅ Yes (must be unique)
  "visibility": "public|private", // ✅ Yes
  "group_id": number,          // ✅ Yes (or null)
  "tags": ["tag1", "tag2"]     // ✅ Yes (via separate endpoint)
}
```

**What you CANNOT update:**
```json
{
  "file": "...",              // ❌ No - upload new version instead
  "width": 1920,              // ❌ No - read-only (from file)
  "height": 1080,             // ❌ No - read-only (from file)
  "duration": 120,            // ❌ No - read-only (from file)
  "codec": "h264",            // ❌ No - read-only (from file)
  "file_size": 1000000        // ❌ No - read-only (from file)
}
```

**File attributes are READ-ONLY** - they come from the actual file and can't be manually changed.

### Delete

**Endpoint:** `DELETE /api/videos/:slug`

**What it does:**
1. Check permissions (owner or editor)
2. Delete database record
3. Delete all associated files:
   - Original file
   - Thumbnails
   - Converted versions (WebP, etc.)
4. Remove from junction tables (video_tags, etc.)
5. Return success

**Cascade deletion:**
```
DELETE video
├── Removes from video_tags
├── Deletes original.mp4
├── Deletes thumbnail.jpg
└── Deletes poster.jpg
```

---

## 🔄 Automatic Processing (Upload Only)

### What Happens Automatically at Upload

#### Images
```
Upload: photo.jpg
↓
Automatic Processing:
├── Generate thumbnail (200x150)
├── Convert to WebP (optimization)
├── Extract EXIF data (camera, date, location)
├── Calculate dimensions (width, height)
├── Store file size
└── Generate unique slug
↓
Files on Disk:
├── /images/photo-abc123.jpg (original)
├── /images/photo-abc123.webp (optimized)
└── /thumbnails/photo-abc123.webp (thumbnail)
```

**Why WebP conversion?**
- ✅ Smaller file sizes (30-50% reduction)
- ✅ Faster loading
- ✅ Better for web delivery
- ✅ Original preserved for download

#### Videos
```
Upload: video.mp4
↓
Automatic Processing:
├── Generate thumbnail (frame at 2 seconds)
├── Generate poster (frame at 5 seconds)
├── Extract metadata (duration, resolution, codec, fps)
├── Calculate file size
└── Generate unique slug
↓
Files on Disk:
├── /videos/video-xyz789.mp4 (original)
├── /thumbnails/video-xyz789.jpg (thumbnail)
└── /posters/video-xyz789.jpg (poster)
```

**Optional (if enabled):**
- HLS transcoding for adaptive streaming
- Multiple quality versions

### No Re-Processing on Update

```
User updates video title:
PUT /api/videos/my-video
{ "title": "New Title" }

What happens:
✅ Database: UPDATE videos SET title = 'New Title'
❌ NO file re-processing
❌ NO thumbnail regeneration
❌ NO re-encoding

Result: Instant update (just metadata)
```

---

## 🎨 Example Workflows

### Workflow 1: Image Upload → Edit Metadata

```bash
# 1. Upload image (automatic processing happens)
POST /api/images
File: photo.jpg (5MB)
Body: {
  "title": "Product Photo",
  "description": "Main product image",
  "visibility": "private"
}

# Server automatically:
# - Generates thumbnail
# - Converts to WebP
# - Extracts EXIF
# - Stores files

Response:
{
  "slug": "product-photo-abc123",
  "width": 4000,
  "height": 3000,
  "file_size": 5242880,
  "mime_type": "image/jpeg",
  "webp_url": "/images/product-photo-abc123.webp"
}

# 2. Later: Update metadata (no file changes)
PUT /api/images/product-photo-abc123
{
  "title": "Hero Product Image",
  "description": "Updated description",
  "visibility": "public"
}

# Server updates:
# - Only database record
# - Files untouched

Response: 200 OK

# 3. Add tags (metadata only)
POST /api/images/product-photo-abc123/tags
{
  "tags": ["product", "hero", "marketing"]
}

# Server updates:
# - Junction table (image_tags)
# - Files untouched

Response: 200 OK
```

### Workflow 2: Video Upload → Edit Metadata → Delete

```bash
# 1. Upload video
POST /api/videos
File: tutorial.mp4 (50MB)
Body: {
  "title": "Tutorial Video",
  "group_id": 42
}

# Server automatically processes

Response:
{
  "slug": "tutorial-video-xyz789",
  "duration": 300,
  "width": 1920,
  "height": 1080,
  "thumbnail_url": "/thumbnails/tutorial-video-xyz789.jpg"
}

# 2. Update metadata
PUT /api/videos/tutorial-video-xyz789
{
  "title": "Introduction Tutorial",
  "description": "Learn the basics"
}

# Files unchanged, only metadata updated

# 3. Delete
DELETE /api/videos/tutorial-video-xyz789

# Server deletes:
# - Database record
# - Original video file
# - Thumbnail
# - Poster
# - Tag associations
```

---

## 📊 CRUD vs Processing Matrix

| Operation | Endpoint | File Processing? | Metadata Change? | When? |
|-----------|----------|------------------|------------------|-------|
| **Upload (Create)** | POST /api/videos | ✅ Yes (automatic) | ✅ Yes | Upload time |
| **Get (Read)** | GET /api/videos/:slug | ❌ No | ❌ No | Anytime |
| **Update Metadata** | PUT /api/videos/:slug | ❌ No | ✅ Yes | Anytime |
| **Delete** | DELETE /api/videos/:slug | ✅ Yes (cleanup) | ✅ Yes | Deletion time |
| **Add Tags** | POST /api/videos/:slug/tags | ❌ No | ✅ Yes | Anytime |

**Key Takeaway:** File processing only happens at **upload** and **delete** (cleanup).

---

## 🚫 What's NOT in CRUD

### Image Editing (Future Feature - Separate from CRUD)

```
NOT in basic CRUD:
❌ Crop image
❌ Rotate image
❌ Apply filters
❌ Resize to specific dimensions
❌ Change format (JPG → PNG)
❌ Compress further
```

**If needed:** Separate "Image Editing" feature (Phase 6+) with dedicated endpoints like:
```
POST /api/images/:slug/edit
{
  "operation": "crop",
  "x": 100,
  "y": 100,
  "width": 800,
  "height": 600
}
```

### Video Editing (Future Feature - Separate from CRUD)

```
NOT in basic CRUD:
❌ Trim video
❌ Add watermark
❌ Change resolution
❌ Re-encode
❌ Extract clips
❌ Add subtitles manually
```

**If needed:** Separate "Video Editing" feature (Phase 7+).

---

## ✅ Summary

### What CRUD Operations Do

**Create (Upload):**
- ✅ Accept file
- ✅ **Auto-process** (thumbnails, WebP, metadata extraction)
- ✅ Store files
- ✅ Save metadata

**Read:**
- ✅ Return metadata
- ✅ Return file URLs

**Update:**
- ✅ Update title, description, slug
- ✅ Change visibility, group
- ✅ Add/remove tags
- ❌ **NO file modification**

**Delete:**
- ✅ Remove database record
- ✅ Delete all associated files
- ✅ Clean up relationships

### Automatic Processing

**Happens once at upload:**
- ✅ Image → WebP conversion ✅
- ✅ Thumbnail generation ✅
- ✅ Metadata extraction ✅
- ❌ Manual rescaling ❌
- ❌ Manual format changes ❌

### Your Understanding is CORRECT

✅ CRUD = Metadata management (title, description, slug, tags, visibility, group)  
✅ Upload = Automatic processing (WebP, thumbnails, metadata)  
✅ Update = NO file changes, only metadata  
✅ Delete = Remove everything  

**This matches the MASTER_PLAN perfectly!** 🎉

---

**Document Version:** 1.0  
**Last Updated:** February 2026  
**Status:** ✅ Aligned with implementation
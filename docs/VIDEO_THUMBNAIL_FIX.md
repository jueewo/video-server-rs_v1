# Video Thumbnail Fix

## Problem

Videos in the "All Media" view were showing placeholder icons instead of their actual thumbnails, even though thumbnail files existed in the video folders.

## Root Cause

The system underwent a migration from separate tables (`videos`, `images`, `documents`) to a unified `media_items` table. However, the video processing code was only updating the legacy `videos` table with thumbnail URLs, not the new `media_items` table that the "All Media" view queries.

### Architecture Context

- **Legacy System**: Separate tables for each media type
  - `videos` table for video metadata
  - `images` table for image metadata  
  - `documents` table for document metadata

- **New System**: Unified `media_items` table
  - Single table for all media types
  - Used by the media-hub crate for the "All Media" view
  - Simplifies cross-media queries and UI

- **Migration State**: The system is in a transitional state where:
  - Both legacy and unified tables exist
  - Video processing still updates legacy `videos` table
  - Media Hub queries unified `media_items` table
  - This mismatch caused thumbnails to be missing

## Solution

### 1. Update Video Processing Code

Modified `video-server-rs_v1/crates/video-manager/src/processing.rs` to update both tables:

```rust
// Update video record (legacy table)
sqlx::query(/* ... */)
    .execute(&context.pool)
    .await?;

// Update media_items table (unified table) with thumbnail URL
sqlx::query(
    r#"
    UPDATE media_items
    SET
        thumbnail_url = ?,
        preview_url = ?,
        file_size = ?,
        status = 'active'
    WHERE slug = ? AND media_type = 'video'
    "#,
)
.bind(&thumbnail_url)
.bind(&hls_url)
.bind(metadata.file_size as i64)
.bind(&context.slug)
.execute(&context.pool)
.await?;
```

This ensures that future video uploads will have thumbnails in both tables.

### 2. Migration Script for Existing Videos

Created `update_video_thumbnails.sh` to fix existing videos:

```bash
#!/bin/bash
# Updates thumbnail URLs for all existing videos in media_items table

sqlite3 media.db <<EOF
UPDATE media_items
SET thumbnail_url = '/hls/' || slug || '/thumbnail.webp',
    preview_url = '/hls/' || slug || '/master.m3u8'
WHERE media_type = 'video'
AND (thumbnail_url IS NULL OR thumbnail_url = '');
EOF
```

## Thumbnail URL Pattern

Videos use the HLS proxy endpoint for thumbnail serving:

```
/hls/{slug}/thumbnail.webp
```

This endpoint:
- Serves files directly from the video's storage folder
- Handles vault-based storage resolution
- Works with access control (access codes, authentication)
- Supports both vault and user-based storage paths

## Storage Location

Thumbnails are stored in the video folder:

```
storage/vaults/{vault_id}/videos/{slug}/thumbnail.webp
```

Or for user-based storage:

```
storage/users/{user_id}/videos/{slug}/thumbnail.webp
```

## Verification

After applying the fix:

```bash
# Check media_items table has thumbnail URLs
sqlite3 media.db "SELECT slug, thumbnail_url FROM media_items WHERE media_type='video';"

# Verify thumbnail files exist
find storage -name "thumbnail.webp" -path "*/videos/*"
```

## Future Considerations

### Complete Migration Path

To fully migrate to the unified system:

1. **Phase 1** ✅ - Create unified `media_items` table
2. **Phase 2** ✅ - Update queries to read from `media_items`
3. **Phase 3** ✅ - Update write operations (this fix)
4. **Phase 4** 📋 - Migrate all legacy data to `media_items`
5. **Phase 5** 📋 - Remove legacy tables and code

### Thumbnail Generation

Current process:
1. Video uploaded
2. FFmpeg extracts frame at 10% duration
3. Converted to WebP format
4. Saved as `thumbnail.webp` in video folder
5. Database updated with `/hls/{slug}/thumbnail.webp` URL

### Fallback Chain

The template handles missing thumbnails gracefully:

```html
{% match item.item.thumbnail_url() %}
{% when Some with (url) %}
    <img src="{{ url }}" ... />
{% when None %}
    <!-- Show emoji icon based on media type -->
    <div class="...">🎬</div>
{% endmatch %}
```

## Related Files

- `crates/video-manager/src/processing.rs` - Video processing pipeline
- `crates/media-hub/templates/media_list_tailwind.html` - Media list view
- `crates/media-hub/src/models.rs` - UnifiedMediaItem model
- `crates/common/src/models/media_item.rs` - MediaItem struct
- `update_video_thumbnails.sh` - Migration script

## Testing

To test the fix:

1. Upload a new video - should automatically have thumbnail
2. View "All Media" page - existing videos should show thumbnails
3. Check browser console - no 404 errors for thumbnail URLs
4. Verify HLS proxy serves thumbnails correctly

```bash
# Test thumbnail endpoint
curl -I http://localhost:8080/hls/test-video/thumbnail.webp
```

## Notes

- This fix maintains backward compatibility with both legacy and unified tables
- The HLS proxy handler already supports serving thumbnails - no changes needed there
- Thumbnail files are generated as WebP for optimal size/quality
- Future uploads will automatically populate both tables
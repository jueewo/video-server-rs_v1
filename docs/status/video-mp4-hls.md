# Video System Status - MP4 + HLS Support

## Overview

Implemented dual video format support allowing users to choose between:
- **HLS (transcoded)**: Adaptive streaming with multiple quality levels
- **MP4 (direct)**: Direct playback without transcoding

## Changes Made

### 1. Database
- **File**: `migrations/017_video_type.sql`
- Added `video_type` column to `media_items` table
- Values: `'hls'` (default) or `'mp4'`

### 2. Upload Form UI
- **File**: `crates/media-manager/templates/media_upload.html`
- Added "Transcode for streaming" checkbox
- Only shows for video files
- Default: checked (HLS mode)
- Unchecked = MP4 mode

### 3. Thumbnail Conversion
- **File**: `crates/video-manager/src/processing.rs`
- Added `convert_thumbnail_to_webp()` function
- Converts FFmpeg-generated JPEG thumbnails to WebP
- Better compression, consistent with image handling

### 4. Dependencies
- **File**: `crates/video-manager/Cargo.toml`
- Added `image` crate with `webp` feature

### 5. Video Serving
- **Files**: 
  - `crates/media-manager/src/routes.rs`
  - `crates/media-manager/src/serve.rs`
- Added route: `GET /media/{slug}/video.mp4`
- Added `serve_video_mp4()` handler
- Checks `video_type` column to serve correct format

## How It Works

### Upload Flow
1. User selects video file
2. "Transcode for streaming" checkbox appears (checked by default)
3. **Checked (HLS)**: Video goes through full transcoding pipeline → HLS segments + m3u8
4. **Unchecked (MP4)**: Video saved directly as MP4 → thumbnail generated via FFmpeg

### Storage Structure
```
storage/vaults/{vault_id}/video/{slug}/
# MP4 mode
├── video.mp4
└── thumbnail.webp

# HLS mode  
├── master.m3u8
├── 1080p/index.m3u8
├── 720p/index.m3u8
├── thumbnail.webp
└── poster.jpg (legacy)
```

### Playback
- Player auto-detects format:
  - HLS: Uses hls.js → serves `/hls/{slug}/master.m3u8`
  - MP4: Native HTML5 video → serves `/media/{slug}/video.mp4`

## Files Modified

| File | Change |
|------|--------|
| `migrations/017_video_type.sql` | NEW - database migration |
| `crates/media-manager/templates/media_upload.html` | Added checkbox |
| `crates/media-manager/src/upload.rs` | Parse transcode option |
| `crates/video-manager/Cargo.toml` | Added image dependency |
| `crates/video-manager/src/processing.rs` | JPEG→WebP conversion |
| `crates/media-manager/src/routes.rs` | Added MP4 route |
| `crates/media-manager/src/serve.rs` | Added serve_video_mp4 |

## Register Video

- **Unchanged** - Register video still only works for HLS folders
- Users can only register videos that have `master.m3u8` in the folder

## Remaining Tasks

1. Run database migration
2. Implement actual MP4 upload handling in `upload.rs`:
   - Save MP4 directly to storage
   - Generate thumbnail via FFmpeg
   - Set `video_type = 'mp4'` in database

---

*Last updated: 2026-02-27*

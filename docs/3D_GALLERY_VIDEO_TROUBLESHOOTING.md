# 3D Gallery Video Troubleshooting Guide

## Problem: Videos Not Showing in 3D Gallery

This document explains why videos might not appear in the 3D gallery and how to fix it.

## Root Cause Analysis

After investigation, we discovered that **videos ARE being returned by the API**, but they fail to stream due to **insufficient access permissions**.

### The Issue

The 3D gallery uses HLS (HTTP Live Streaming) to play videos. HLS requires downloading video segments, which needs **Download** permission level, not just **Read** permission.

**Key Finding:**
- API endpoint `/api/3d/gallery?code=gallery3d` returns videos correctly ✅
- HLS endpoint `/hls/:slug/master.m3u8?code=gallery3d` returns 401 Unauthorized ❌
- Access code `gallery3d` had permission level `read` (insufficient for video streaming)

## Solution

Update the access code permission level from `read` to `download`:

```sql
UPDATE access_codes 
SET permission_level = 'download' 
WHERE code = 'gallery3d';
```

### Quick Fix Script

```bash
cd video-server-rs_v1
sqlite3 media.db "UPDATE access_codes SET permission_level = 'download' WHERE code = 'gallery3d';"
```

Then restart the server:

```bash
pkill video-server-rs
cargo run
```

## Understanding Permission Levels

The access control system has a hierarchical permission structure:

```
Admin (5)     - Full administrative control
  ↓ includes
Delete (4)    - Can delete resources
  ↓ includes
Edit (3)      - Can modify resources
  ↓ includes
Download (2)  - Can download and stream resources ← Required for HLS video
  ↓ includes
Read (1)      - Can view only (no streaming)
```

### Why Videos Need Download Permission

HLS video streaming works by:
1. Client fetches `master.m3u8` playlist
2. Client fetches multiple `.ts` segment files
3. Segments are progressively downloaded and played

Each segment download is checked against access permissions. Since downloading segments is effectively "downloading" the video, the system requires `Download` permission level.

## Diagnostic Steps

### 1. Verify Media Items in Database

```bash
sqlite3 media.db "SELECT slug, title, media_type FROM media_items WHERE media_type = 'video';"
```

Expected output:
```
bbb|Big Buck Bunny|video
lesson1|Lesson 1|video
welcome|Welcome|video
webconjoint|Web Conjoint|video
test-demo-video|Test Demo Video|video
```

### 2. Check Access Code Permissions

```bash
sqlite3 media.db "SELECT code, permission_level, is_active FROM access_codes WHERE code = 'gallery3d';"
```

Expected output:
```
gallery3d|download|1
```

**Important:** Permission level must be `download` or higher, NOT `read`.

### 3. Verify Access Code Permissions Mapping

```bash
sqlite3 media.db "
SELECT ac.code, acp.media_type, acp.media_slug 
FROM access_codes ac 
INNER JOIN access_code_permissions acp ON ac.id = acp.access_code_id 
WHERE ac.code = 'gallery3d' AND acp.media_type = 'video';
"
```

Expected output should include all videos:
```
gallery3d|video|bbb
gallery3d|video|lesson1
gallery3d|video|test-demo-video
gallery3d|video|webconjoint
gallery3d|video|welcome
```

### 4. Check Video Files Exist

```bash
ls -la storage/vaults/vault-90b0d507/videos/
```

Expected output:
```
drwxr-xr-x  5 user  staff  160 Feb 12 11:53 bbb
drwxr-xr-x  5 user  staff  160 Feb 12 11:53 lesson1
drwxr-xr-x  5 user  staff  160 Feb 12 11:53 test-demo-video
drwxr-xr-x  5 user  staff  160 Feb 12 11:53 webconjoint
drwxr-xr-x  5 user  staff  160 Feb 12 11:53 welcome
```

### 5. Verify HLS Files

```bash
ls -la storage/vaults/vault-90b0d507/videos/bbb/
```

Expected output:
```
-rw-r--r--  1 user  staff  3717 Dec 19 17:29 master.m3u8
drwxr-xr-x  108 user  staff  3456 Feb 12 11:53 segments
-rw-r--r--  1 user  staff  9814 Dec 19 17:29 thumbnail.webp
```

### 6. Test API Response

```bash
curl -s "http://localhost:3000/api/3d/gallery?code=gallery3d" | jq '.items[] | select(.media_type == "video") | {id, title, url}'
```

Expected output (videos should be present):
```json
{
  "id": 27,
  "title": "Lesson 1",
  "url": "/hls/lesson1/master.m3u8?code=gallery3d"
}
{
  "id": 28,
  "title": "Big Buck Bunny",
  "url": "/hls/bbb/master.m3u8?code=gallery3d"
}
...
```

### 7. Test HLS Endpoint (Critical Test)

```bash
curl -s -o /dev/null -w "%{http_code}\n" "http://localhost:3000/hls/bbb/master.m3u8?code=gallery3d"
```

Expected output:
```
200
```

**If you get `401`:** The access code doesn't have sufficient permissions. Update to `download` level.

**If you get `404`:** The video file doesn't exist or vault path is incorrect.

**If you get `500`:** Server error - check logs.

### 8. Test Actual HLS Content

```bash
curl -s "http://localhost:3000/hls/bbb/master.m3u8?code=gallery3d"
```

Expected output (HLS playlist):
```
#EXTM3U
#EXT-X-VERSION:6
#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360
playlist_0.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=1400000,RESOLUTION=842x480
playlist_1.m3u8
...
```

## Common Issues and Solutions

### Issue 1: 401 Unauthorized on HLS Endpoint

**Symptoms:**
- API returns videos
- Videos appear in gallery but don't play
- HLS endpoint returns 401

**Cause:** Access code has `read` permission but needs `download` permission.

**Solution:**
```bash
sqlite3 media.db "UPDATE access_codes SET permission_level = 'download' WHERE code = 'gallery3d';"
```

### Issue 2: Videos Not in Access Code Permissions

**Symptoms:**
- API returns empty items array or missing videos
- Only images show up

**Cause:** `access_code_permissions` table doesn't have entries for the videos.

**Solution:** Add permissions for each video:
```sql
INSERT INTO access_code_permissions (access_code_id, media_type, media_slug)
SELECT ac.id, 'video', mi.slug
FROM access_codes ac, media_items mi
WHERE ac.code = 'gallery3d' 
  AND mi.media_type = 'video'
  AND NOT EXISTS (
    SELECT 1 FROM access_code_permissions acp 
    WHERE acp.access_code_id = ac.id 
      AND acp.media_slug = mi.slug
  );
```

### Issue 3: Video Files Missing After Server Copy

**Symptoms:**
- Database has video records
- HLS endpoint returns 404
- Files don't exist in storage directory

**Cause:** When copying `media.db` to server, forgot to copy `storage/` directory.

**Solution:**
1. Copy storage directory from local to server:
   ```bash
   rsync -av storage/ user@server:/path/to/video-server-rs_v1/storage/
   ```

2. Verify permissions:
   ```bash
   chmod -R 755 storage/
   ```

### Issue 4: Wrong Vault ID

**Symptoms:**
- Database has video records with vault_id
- Files exist but in different vault
- HLS returns 404

**Cause:** Database vault_id doesn't match actual storage structure.

**Solution:** Check actual vault structure:
```bash
find storage/vaults -name "*.m3u8" -type f
```

Update database if needed:
```sql
UPDATE media_items 
SET vault_id = 'vault-ACTUAL-ID' 
WHERE media_type = 'video';
```

### Issue 5: Server Not Restarted After Permission Change

**Symptoms:**
- Permission level updated in database
- HLS still returns 401

**Cause:** Some systems may cache access decisions (though the current implementation doesn't).

**Solution:**
```bash
pkill video-server-rs
cargo run
```

## Verification Checklist

After making changes, verify everything works:

- [ ] Access code exists and is active
- [ ] Access code has `download` or higher permission level
- [ ] Access code permissions include video slugs
- [ ] Video records exist in `media_items` table
- [ ] Video files exist in storage directory
- [ ] HLS endpoint returns 200 status code
- [ ] HLS endpoint returns valid m3u8 content
- [ ] 3D gallery shows videos in browser
- [ ] Videos play when clicked in gallery

## Testing in Browser

1. Open browser developer console (F12)
2. Navigate to: `http://localhost:3000/3d?code=gallery3d`
3. Check Console tab for errors
4. Check Network tab for failed requests (especially `.ts` files)
5. Look for 401 errors on HLS segment requests

## Server Logs

The server logs access control decisions. Look for:

```
Access denied to HLS stream: reason="Access key grants Read permission, but Download was requested"
```

Or:

```
Access granted to HLS stream: layer=AccessKey
```

## Architecture Notes

The video streaming flow:

```
Browser
  ↓ requests
3D Gallery API (/api/3d/gallery?code=X)
  ↓ queries database with JOIN
Returns video list with HLS URLs
  ↓ browser clicks video
HLS Endpoint (/hls/:slug/master.m3u8?code=X)
  ↓ checks access control
Access Control System (4 layers)
  ↓ Layer 2: Access Key (checks permission level)
Returns m3u8 playlist or 401
  ↓ browser requests segments
HLS Endpoint (/hls/:slug/segments/*.ts?code=X)
  ↓ each segment checked again
Returns video segments or 401
```

**Key Point:** Every HLS request goes through access control. The access code must grant `download` permission for streaming to work.

## Database Schema Reference

### access_codes
```sql
- id: INTEGER PRIMARY KEY
- code: TEXT (e.g., "gallery3d")
- permission_level: TEXT (must be "download" or higher for video)
- is_active: BOOLEAN (must be 1)
- expires_at: TEXT (optional, check if not expired)
```

### access_code_permissions
```sql
- id: INTEGER PRIMARY KEY
- access_code_id: INTEGER (FK to access_codes.id)
- media_type: TEXT ("video" or "image")
- media_slug: TEXT (matches media_items.slug)
```

### media_items
```sql
- id: INTEGER PRIMARY KEY
- slug: TEXT (unique identifier)
- media_type: TEXT ("video", "image", or "document")
- vault_id: TEXT (references storage vault)
- filename: TEXT
- title: TEXT
```

## Quick Reference Commands

### Check everything at once:
```bash
echo "=== Media Items ==="
sqlite3 media.db "SELECT COUNT(*) as videos FROM media_items WHERE media_type='video';"

echo "=== Access Code ==="
sqlite3 media.db "SELECT code, permission_level, is_active FROM access_codes WHERE code='gallery3d';"

echo "=== Permissions ==="
sqlite3 media.db "SELECT COUNT(*) as video_perms FROM access_code_permissions acp JOIN access_codes ac ON acp.access_code_id=ac.id WHERE ac.code='gallery3d' AND acp.media_type='video';"

echo "=== HLS Test ==="
curl -s -o /dev/null -w "Status: %{http_code}\n" "http://localhost:3000/hls/bbb/master.m3u8?code=gallery3d"
```

## Prevention

When creating new access codes for 3D gallery:

1. **Always use `download` permission level** for galleries that include videos
2. Add permissions for all media items you want to display
3. Test HLS endpoint before deploying
4. Document which galleries require which permission levels

### Creating New Access Codes Properly

```sql
-- Create access code with download permission
INSERT INTO access_codes (code, description, permission_level, is_active)
VALUES ('mygallery', 'My 3D Gallery', 'download', 1);

-- Get the access code ID
SELECT id FROM access_codes WHERE code = 'mygallery';

-- Add all videos to the access code (assuming ID = 42)
INSERT INTO access_code_permissions (access_code_id, media_type, media_slug)
SELECT 42, 'video', slug FROM media_items WHERE media_type = 'video';

-- Add specific images
INSERT INTO access_code_permissions (access_code_id, media_type, media_slug)
VALUES (42, 'image', 'logo'), (42, 'image', 'banner');
```

## Related Documentation

- [ACCESS_CODE_DECISION_GUIDE.md](../ACCESS_CODE_DECISION_GUIDE.md) - Access code system overview
- [PERMISSION_MANAGEMENT_GUIDE.md](../PERMISSION_MANAGEMENT_GUIDE.md) - Permission levels explained
- [VIDEO_MANAGEMENT_GUIDE.md](../VIDEO_MANAGEMENT_GUIDE.md) - Video upload and management

---

**Last Updated:** 2024-02-16  
**Issue Fixed:** Videos not showing in 3D gallery due to insufficient permission level  
**Solution:** Changed access code permission from `read` to `download`

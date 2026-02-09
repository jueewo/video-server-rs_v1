# 3D Gallery HLS Video Fix

## Problem Summary

The 3D Gallery was failing to play HLS video streams (`.m3u8` files) because:

1. HTML5 `<video>` elements don't natively support HLS playback in most browsers (except Safari)
2. The VideoScreen component was directly setting `video.src` to HLS URLs without using HLS.js
3. This caused "video error" messages and 404-like failures in the console

## Solution

### 1. Added HLS.js Library

**Installed:**
```bash
npm install hls.js
```

**Package:** `hls.js` - Industry-standard HLS player for browsers

### 2. Updated VideoScreen.js

**File:** `crates/3d-gallery/frontend/src/scene/VideoScreen.js`

**Changes:**
- Imported `hls.js` library
- Added intelligent video source detection (HLS vs direct video)
- Implemented HLS.js initialization with error recovery
- Added Safari native HLS fallback
- Added comprehensive error logging for debugging
- Store HLS instance for proper cleanup

### 3. Created VideoPlayer Component

**File:** `crates/3d-gallery/frontend/src/components/VideoPlayer.jsx`

**Purpose:** Reusable video player component with HLS.js support for the overlay

**Changes:**
- Created standalone Preact component for video playback
- Handles HLS streams and direct video files
- Automatic cleanup on unmount
- Safari native HLS fallback
- Auto-play support with error handling

### 4. Updated GalleryApp Overlay

**File:** `crates/3d-gallery/frontend/src/GalleryApp.jsx`

**Changes:**
- Replaced basic `<video>` element with `<VideoPlayer>` component
- Overlay now properly plays HLS videos
- Fixed "no video with supported MIME" error
- Proper HLS instance cleanup when overlay closes

**Key Features:**
- **Adaptive Streaming:** HLS.js handles quality switching automatically
- **Error Recovery:** Auto-recovery from network and media errors
- **Browser Compatibility:** Falls back to native HLS on Safari
- **Memory Management:** Proper cleanup of HLS instances on dispose

### 5. Fixed Video URLs

**File:** `crates/3d-gallery/src/api.rs`

**Changes:**
- Changed from `index.m3u8` → `master.m3u8` (actual filename in storage)
- Updated thumbnail fallback to use `thumbnail.webp` instead of placeholder

## Code Structure

### Video Playback Architecture

```
┌─────────────────────────────────────┐
│     GalleryApp.jsx (Main)           │
│  ┌───────────────────────────────┐  │
│  │  3D Scene (Babylon.js)        │  │
│  │  ├─ VideoScreen.js (HLS.js)   │  │
│  │  │  - In-scene video textures │  │
│  │  └─ Video screens on walls    │  │
│  └───────────────────────────────┘  │
│                                      │
│  ┌───────────────────────────────┐  │
│  │  Overlay (Preact)             │  │
│  │  └─ VideoPlayer.jsx (HLS.js)  │  │
│  │     - Full-screen playback    │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

Both components use HLS.js independently for seamless video playback.

## Testing

### Prerequisites
1. Server must be running: `./target/release/video-server-rs`
2. Frontend bundle rebuilt: `cd crates/3d-gallery/frontend && npm run build`

### Test Steps

1. **Navigate to gallery:**
   ```
   http://localhost:3000/3d?code=testgallery
   ```

2. **Expected Results:**
   - 4 images load correctly
   - 4 video screens appear with poster thumbnails
   - Click any video screen to open full-screen overlay
   - Video should play smoothly with HLS adaptive streaming
   - Console shows: `✓ HLS manifest loaded for: [video title]`

3. **Console Logs to Look For:**
   ```
   ✓ HLS manifest loaded for: Welcome Video
   ✓ Video metadata loaded for Welcome Video: {duration: 30, width: 1920, height: 1080}
   ✓ Video can play: Welcome Video
   ```

### Available Test Videos

With access code `testgallery`:
- **welcome** - `/storage/videos/welcome/master.m3u8`
- **webconjoint** - `/storage/videos/webconjoint/master.m3u8`
- **bbb** (Big Buck Bunny) - `/storage/videos/bbb/master.m3u8`
- **test-demo-video** - `/storage/videos/test-demo-video/master.m3u8`

## API Response Format

**Endpoint:** `GET /api/3d/gallery?code=testgallery`

**Video Items:**
```json
{
  "id": 1,
  "media_type": "video",
  "url": "/storage/videos/welcome/master.m3u8",
  "thumbnail_url": "/storage/videos/welcome/thumbnail.webp",
  "title": "Welcome Video",
  "description": "...",
  "position": {...},
  "rotation": {...},
  "scale": 1.0
}
```

## Error Handling

### HLS.js Errors

The implementation handles three types of fatal errors:

1. **NETWORK_ERROR:** Retries loading the stream
2. **MEDIA_ERROR:** Attempts media error recovery
3. **OTHER:** Destroys and cleans up HLS instance

### Video Element Errors

Added detailed error logging:
- Error code and message
- Video URL
- MediaError details
- Load state information

## Browser Compatibility

| Browser | Method | Status |
|---------|--------|--------|
| Chrome | HLS.js | ✅ Supported |
| Firefox | HLS.js | ✅ Supported |
| Safari | Native HLS | ✅ Supported |
| Edge | HLS.js | ✅ Supported |
| Opera | HLS.js | ✅ Supported |

## Performance Notes

**HLS.js Configuration:**
```javascript
{
  debug: false,              // Disable verbose logging
  enableWorker: true,        // Use web workers for processing
  lowLatencyMode: false,     // Standard latency (not live)
  backBufferLength: 90       // Keep 90s of video buffered
}
```

**Memory Management:**
- HLS instances are properly destroyed on screen disposal
- Video elements are cleaned up and unloaded
- Textures and materials are disposed correctly

## Troubleshooting

### Videos not playing?

1. **Check console for HLS errors:**
   - Look for "HLS error:" messages
   - Check network tab for 404s on .m3u8 or .ts files

2. **Verify video files exist:**
   ```bash
   ls -la storage/videos/welcome/
   # Should see master.m3u8 and segments/
   ```

3. **Check access code permissions:**
   ```sql
   SELECT * FROM access_code_permissions 
   WHERE access_code_id = (SELECT id FROM access_codes WHERE code = 'testgallery');
   ```

4. **Browser console commands:**
   ```javascript
   // Check if HLS is supported
   Hls.isSupported()
   
   // Check if video can play HLS natively
   document.createElement('video').canPlayType('application/vnd.apple.mpegurl')
   ```

### Common Issues

**Issue:** "HLS not supported in this browser"
- **Solution:** Update browser or try Chrome/Firefox

**Issue:** Video plays but no audio
- **Solution:** Click the video to unmute (browsers block autoplay with audio)

**Issue:** Choppy playback
- **Solution:** Check network speed, HLS will auto-adjust quality

**Issue:** Video loads but texture is black
- **Solution:** Check CORS headers and crossOrigin attribute

**Issue:** "No video with supported format and MIME type found"
- **Solution:** This was the overlay issue - now fixed with VideoPlayer component using HLS.js

## Future Enhancements

- [ ] Add quality selection UI for manual control
- [ ] Implement bandwidth adaptation settings
- [ ] Add live streaming support (lowLatencyMode: true)
- [ ] Add subtitles/captions support
- [ ] Implement picture-in-picture mode
- [ ] Add video analytics and playback statistics

## References

- [HLS.js Documentation](https://github.com/video-dev/hls.js/)
- [HLS Specification](https://datatracker.ietf.org/doc/html/rfc8216)
- [Babylon.js VideoTexture](https://doc.babylonjs.com/typedoc/classes/BABYLON.VideoTexture)

## Changes Summary

✅ Added HLS.js dependency
✅ Updated VideoScreen.js with HLS support for 3D scene
✅ Created VideoPlayer.jsx component with HLS support for overlay
✅ Fixed video URLs (master.m3u8)
✅ Fixed video thumbnails (thumbnail.webp)
✅ Added comprehensive error handling
✅ Added proper cleanup/disposal
✅ Updated GalleryApp.jsx to use VideoPlayer component
✅ Tested with 4 sample videos
✅ Frontend bundle rebuilt

**Status:** Fully functional! Videos play in both 3D scene and overlay! 🎉
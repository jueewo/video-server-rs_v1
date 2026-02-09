# 3D Gallery - Video Playback Test Checklist

## ✅ Pre-Test Setup

- [ ] Server is running: `./target/release/video-server-rs`
- [ ] Frontend bundle rebuilt: `cd crates/3d-gallery/frontend && npm run build`
- [ ] Browser console is open (F12) for monitoring
- [ ] Test URL ready: `http://localhost:3000/3d?code=testgallery`

## ✅ Test 1: Gallery Loads

- [ ] Navigate to `http://localhost:3000/3d?code=testgallery`
- [ ] Loading screen appears briefly
- [ ] 3D gallery room loads successfully
- [ ] Camera controls work (mouse drag, WASD keys)

**Expected:** Gallery loads with walls, floor, ceiling

## ✅ Test 2: Images Display

- [ ] 4 image frames visible on walls
- [ ] Images are right-side-up (not upside down)
- [ ] Images are not mirrored
- [ ] Can see image thumbnails clearly

**Expected Images:**
- Company Logo
- Welcome Banner
- My Profile Pic
- Cluster Demo

## ✅ Test 3: Video Screens Display

- [ ] 4 video screens visible on walls
- [ ] Video poster/thumbnails are visible
- [ ] Screens have black frames (like TV bezels)
- [ ] No console errors about 404 on .m3u8 files

**Expected Videos:**
- Welcome Video
- WebConjoint Teaser Video
- Big Buck Bunny
- test-demo-video

## ✅ Test 4: Video Playback in 3D Scene

- [ ] Click on a video screen in the 3D scene
- [ ] Console shows: `✓ HLS manifest loaded for: [title]`
- [ ] Video plays/pauses on click
- [ ] Video texture updates on the screen mesh
- [ ] Can toggle play/pause by clicking screen again

**Console logs to expect:**
```
✓ HLS manifest loaded for: Welcome Video
✓ Video metadata loaded for Welcome Video
✓ Video can play: Welcome Video
▶ Playing: Welcome Video
```

## ✅ Test 5: Video Overlay (CRITICAL FIX)

- [ ] Click on any video screen
- [ ] Full-screen overlay appears with dark background
- [ ] **Video plays in overlay** (NOT "no supported MIME" error)
- [ ] Console shows: `✓ HLS manifest loaded in overlay`
- [ ] Video controls visible (play/pause, volume, timeline)
- [ ] Video plays smoothly with adaptive quality

**BEFORE FIX:** "No video with supported format and MIME type found"
**AFTER FIX:** Video plays correctly using HLS.js

## ✅ Test 6: Overlay Controls

- [ ] Video auto-plays when overlay opens
- [ ] Can pause/play using video controls
- [ ] Volume slider works
- [ ] Timeline scrubbing works
- [ ] Video title and description visible below player

## ✅ Test 7: Close Overlay

- [ ] Click dark background outside video → Overlay closes
- [ ] Press ESC key → Overlay closes
- [ ] Click "Close" button → Overlay closes
- [ ] Camera control reattaches after closing
- [ ] Console shows: "Camera reattached - overlay closed"

## ✅ Test 8: Image Overlay (Should Still Work)

- [ ] Click on an image frame
- [ ] Image overlay appears
- [ ] High-quality image displayed
- [ ] Can close with click/ESC/button
- [ ] Camera reattaches properly

## ✅ Test 9: Multiple Videos

Test each video individually:

- [ ] **Welcome Video** - plays in overlay with HLS
- [ ] **WebConjoint Teaser** - plays in overlay with HLS
- [ ] **Big Buck Bunny** - plays in overlay with HLS
- [ ] **test-demo-video** - plays in overlay with HLS

## ✅ Test 10: Console - No Errors

Check browser console for:

- [ ] No red errors about 404 on .m3u8 files
- [ ] No "MIME type" errors
- [ ] No "video error" messages
- [ ] Only green ✓ success messages

**Good messages:**
```
✓ HLS manifest loaded for: [title]
✓ Video metadata loaded for [title]
✓ Video can play: [title]
```

**Bad messages (should NOT appear):**
```
❌ HTTP 404 on index.m3u8
❌ Load of media resource failed
❌ No video with supported format and MIME type found
```

## ✅ Test 11: Performance

- [ ] Gallery loads in < 5 seconds
- [ ] Camera movement is smooth (60fps)
- [ ] Video textures update without lag
- [ ] Overlay opens/closes instantly
- [ ] No memory leaks (check browser task manager)

## ✅ Test 12: Browser Compatibility

Test in multiple browsers:

- [ ] **Chrome/Edge** - Uses HLS.js
- [ ] **Firefox** - Uses HLS.js
- [ ] **Safari** - Uses native HLS (should work without HLS.js)

## 🔧 Troubleshooting

### If videos don't play in overlay:

1. Check bundle was rebuilt:
   ```bash
   ls -lh crates/3d-gallery/static/bundle.js
   # Should be ~4.4MB and timestamp should be recent
   ```

2. Hard refresh browser: `Cmd+Shift+R` (Mac) or `Ctrl+Shift+R` (Windows)

3. Check console for HLS errors:
   ```
   HLS error: [type] [details]
   ```

4. Verify video files exist:
   ```bash
   ls storage/videos/welcome/master.m3u8
   curl http://localhost:3000/storage/videos/welcome/master.m3u8
   ```

### If videos show 404:

1. Check API response:
   ```bash
   curl http://localhost:3000/api/3d/gallery?code=testgallery | jq '.items[] | select(.media_type=="video")'
   ```

2. Verify URLs use `master.m3u8` not `index.m3u8`

### If HLS.js not found:

1. Reinstall dependencies:
   ```bash
   cd crates/3d-gallery/frontend
   npm install hls.js
   npm run build
   ```

## 📊 Success Criteria

### PASS if:
- ✅ All 8 media items load (4 images + 4 videos)
- ✅ Videos play in 3D scene as textures
- ✅ Videos play in overlay with controls
- ✅ No MIME type errors in console
- ✅ No 404 errors on .m3u8 files
- ✅ Overlay opens/closes smoothly
- ✅ Camera controls work correctly

### FAIL if:
- ❌ "No video with supported format" error
- ❌ Videos don't play in overlay
- ❌ 404 errors on .m3u8 files
- ❌ Console shows "video error"
- ❌ Overlay doesn't open
- ❌ Camera gets stuck

## 🎉 Expected Final Result

1. **3D Gallery Scene:**
   - 4 images on walls (clickable)
   - 4 video screens on walls (clickable, playing as textures)
   - Smooth camera movement
   - Dynamic ceiling transparency

2. **Video Overlay:**
   - Full-screen video player
   - HLS adaptive streaming
   - Standard video controls
   - Title and description
   - Smooth open/close

3. **Console Output:**
   ```
   ✓ Gallery data loaded
   ✓ Created 4 image frames
   ✓ Created 4 video screens
   ✓ HLS manifest loaded for: Welcome Video
   ✓ HLS manifest loaded for: WebConjoint Teaser Video
   ✓ HLS manifest loaded for: Big Buck Bunny
   ✓ HLS manifest loaded for: test-demo-video
   Video clicked: test-demo-video
   ✓ HLS manifest loaded in overlay
   Camera detached - overlay open
   Camera reattached - overlay closed
   ```

## 📝 Notes

- HLS.js handles adaptive bitrate streaming automatically
- Videos may take 1-2 seconds to start (normal for HLS)
- First video load might be slower (browser caching)
- Clicking video screen toggles play/pause in 3D scene
- Clicking video screen also opens overlay for full viewing

---

**Date:** February 9, 2025
**Version:** v1.0 - HLS Video Fix
**Status:** Ready for Testing ✅
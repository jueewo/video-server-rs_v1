# Quick Test Guide - Video Fixes

## 🚀 What Changed

1. ✅ **Fixed BBB video mirroring** - No more left-right flip
2. ✅ **Lazy loading** - Videos only load when clicked
3. ✅ **Poster images** - Thumbnails show until video plays

## 🧪 Quick Test (2 minutes)

### Step 1: Refresh Browser
```
http://localhost:3000/3d?code=testgallery
```
**Hard refresh:** Cmd+Shift+R (Mac) or Ctrl+Shift+R (Windows)

### Step 2: Check Video Screens
- [ ] 4 video screens visible
- [ ] **Poster images** showing (not black screens)
- [ ] No videos playing automatically
- [ ] No HLS manifest messages in console yet

### Step 3: Test BBB Video Orientation
- [ ] Look at **Big Buck Bunny** screen
- [ ] Click to play it
- [ ] **Check orientation:** Should be normal (not mirrored)
- [ ] Bunny should face right direction
- [ ] Text/logos should be readable (not backwards)

### Step 4: Test Lazy Loading
Open browser **Network tab** (F12 → Network):
- [ ] Gallery loads
- [ ] **No .m3u8 or .ts files** in network requests yet
- [ ] Only poster image requests
- [ ] Click a video screen
- [ ] **Now** see .m3u8 and .ts requests
- [ ] Console shows: `✓ HLS manifest loaded for: [title]`

### Step 5: Test All Videos
Click each video one by one:
- [ ] Welcome Video - plays correctly
- [ ] WebConjoint Teaser - plays correctly  
- [ ] Big Buck Bunny - **not mirrored** ✨
- [ ] test-demo-video - plays correctly

### Step 6: Test Overlay
- [ ] Click any video screen
- [ ] Overlay opens with video player
- [ ] Video plays in overlay
- [ ] Close overlay
- [ ] Video in 3D scene keeps texture

## ✅ Success Criteria

**PASS if:**
- ✅ BBB video NOT mirrored
- ✅ Videos show posters initially
- ✅ No videos load until clicked
- ✅ Clicked videos play smoothly
- ✅ Overlay works correctly

**FAIL if:**
- ❌ BBB video still mirrored
- ❌ Videos load immediately
- ❌ Black screens instead of posters
- ❌ Videos don't play

## 🐛 Troubleshooting

**If BBB still mirrored:**
- Hard refresh browser
- Check bundle timestamp: `ls -lh crates/standalone/3d-gallery/static/bundle.js`
- Should be Feb 9 19:xx

**If videos auto-load:**
- Check network tab
- Should see NO .m3u8 files until click
- If you see them, clear cache and retry

**If no posters show:**
- Check console for texture errors
- Verify thumbnail URLs in API response

## 📊 Expected Console Output

**On Load:**
```
✓ Created 4 video screens
Created video element for: Welcome Video
(no HLS manifest messages yet)
```

**On First Video Click:**
```
Video clicked: Big Buck Bunny
✓ HLS manifest loaded for: Big Buck Bunny
▶ Playing: Big Buck Bunny
Switched to video texture for: Big Buck Bunny
```

---

**Test Time:** ~2 minutes
**Focus:** BBB orientation + lazy loading
**Status:** Ready to test! 🎯

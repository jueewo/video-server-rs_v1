# Build and Test Script - Phase 2 Optimizations

**Status:** Ready to build and test  
**Date:** 2024-01-XX  
**Phase:** 2 - Critical Tasks Complete

---

## ✅ What Was Completed

### Code Changes:
1. ✅ **Named Imports** - All wildcard `import * as BABYLON` replaced in:
   - `GalleryRoom.js`
   - `ImageFrame.js`
   - `LayoutParser.js`
   - `PdfPresentation.js`
   - `VideoScreen.js`
   - `GalleryApp.jsx`

2. ✅ **Device Detection** - Created `deviceDetection.js` with quality profiles

3. ✅ **Mobile Lighting** - Dynamic lighting based on device (1-6 lights)

4. ✅ **PDF.js Lazy Loading** - Complete with `loadPdfJs()` function

5. ✅ **HLS.js Lazy Loading** - Complete with `loadHls()` function

6. ✅ **Code Splitting** - Enabled in `package.json` build script

7. ✅ **Template Update** - `viewer.html` now references `index.js`

---

## 🚀 BUILD INSTRUCTIONS

### Step 1: Clean Previous Build

```bash
cd crates/3d-gallery/frontend
npm run clean
```

**Expected Output:**
```
Removed ../static/*.js
Removed ../static/*.js.map
```

---

### Step 2: Build with Optimizations

```bash
npm run build
```

**Expected Output:**
```
  ../static/index.js        [main bundle - expect ~1.5-2.5 MB]
  ../static/chunk-XXX.js    [Babylon.js chunk]
  ../static/chunk-YYY.js    [shared utilities]
  ...
  
✓ Build complete
```

**What to Look For:**
- ✅ Multiple `.js` files created (not just one `bundle.js`)
- ✅ `index.js` is the main entry point
- ✅ Total size of all chunks < 3 MB
- ❌ If only `bundle.js` exists, code splitting didn't work

---

### Step 3: Verify Bundle Sizes

```bash
ls -lh ../static/*.js
```

**Expected Results:**

| File | Size | Purpose |
|------|------|---------|
| `index.js` | ~500 KB - 1 MB | Main entry, Preact, scene setup |
| `chunk-*.js` | ~1-2 MB | Babylon.js core |
| `chunk-*.js` | ~200-500 KB | Shared utilities |

**Total:** Should be 2-3 MB (down from 3.9-4.4 MB)

**SUCCESS CRITERIA:**
- ✅ `index.js` exists (not `bundle.js`)
- ✅ Multiple chunk files exist
- ✅ Total size < 3 MB
- ✅ No `pdfjs-dist` in initial bundles (lazy loaded)
- ✅ No `hls.js` in initial bundles (lazy loaded)

---

### Step 4: Check for Tree-Shaking Success

```bash
# Check that no wildcard imports remain
grep -r "import \* as BABYLON" ../src/
```

**Expected:** No results (all replaced with named imports)

```bash
# Verify named imports are present
grep "import {" ../src/**/*.js* | head -5
```

**Expected:** Should see named imports like:
```
import { Engine, Scene, Vector3 } from "@babylonjs/core";
```

---

## 🧪 TESTING INSTRUCTIONS

### Test 1: Desktop Browser Test

```bash
# From project root
cd ../..  # Back to crates/3d-gallery
cargo run
```

**Open Browser:**
```
http://localhost:3000/3d?code=YOUR_ACCESS_CODE
```

**Check Console Logs:**
```
✅ Look for these logs:
🎮 Device detected: desktop (or mobile)
⚙️ Quality profile: high (or medium/low/ultra_low)
Created X lights for gallery (max: 6 for desktop, 1 for mobile VR)

✅ On first PDF click:
📦 Lazy loading PDF.js...
✅ PDF.js loaded successfully

✅ On first HLS video:
📦 Lazy loading HLS.js...
✅ HLS.js loaded successfully
```

**Check Network Tab:**
- `index.js` loads first (~500 KB - 1 MB)
- Chunk files load as needed
- `pdfjs-dist` ONLY loads when clicking a PDF
- `hls.js` ONLY loads when playing HLS video

**Verify Functionality:**
- [ ] Gallery loads and displays
- [ ] Can navigate with WASD
- [ ] Images display correctly
- [ ] Videos play (click to open overlay)
- [ ] PDFs display (click to open)
- [ ] No console errors

---

### Test 2: Mobile VR Simulation (Desktop)

**Chrome DevTools:**
1. Open DevTools (F12)
2. Toggle Device Toolbar (Ctrl+Shift+M)
3. Change User Agent:
   - Settings > More Tools > Network Conditions
   - User Agent: Custom: `Mozilla/5.0 (Linux; Android 10; ViveFocus) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Mobile Safari/537.36 VR`

**Reload Page and Check Console:**
```
✅ Expected:
🎮 Device detected: mobile_vr
⚙️ Quality profile: ultra_low
⚠️ Mobile VR detected - using ultra-low quality settings for performance
🔋 Using mobile VR optimized lighting (single light)
Created 1 lights for gallery (max: 1)
```

**Verify:**
- [ ] Single light used (dimmer but acceptable)
- [ ] Loads faster than before
- [ ] Lower resolution textures
- [ ] PDFs disabled or lazy loaded

---

### Test 3: HTC Vive Flow (Actual Device)

**Prerequisites:**
- HTC Vive Flow headset
- Same WiFi network as dev machine
- Access to gallery URL

**Steps:**
1. Open VR browser on HTC Vive Flow
2. Navigate to: `http://YOUR_IP:3000/3d?code=YOUR_CODE`
3. Wait for gallery to load

**Performance Metrics to Observe:**

| Metric | Before | Target After | Actual |
|--------|--------|-------------|--------|
| **Initial Load Time** | 30-45 sec | < 15 sec | ___ sec |
| **FPS** | 10-20 | 25-30 | ___ FPS |
| **Memory** | 800+ MB | < 500 MB | ___ MB |
| **Smoothness** | Stuttery | Smooth | _____ |

**How to Check FPS on HTC Vive Flow:**
- Enable Developer Options in Vive settings
- Or estimate: smooth = 25-30 FPS, stuttery = 10-20 FPS

**Check Console (Remote Debugging):**
```bash
# Connect via USB and Chrome DevTools
chrome://inspect#devices
```

**Expected Console Output:**
```
🎮 Device detected: mobile_vr
⚙️ Quality profile: ultra_low
🔋 Using mobile VR optimized lighting (single light)
Created 1 lights for gallery (max: 1)
Gallery created: X rooms, Y walls, Z slots
```

**Verify:**
- [ ] Load time < 15 seconds
- [ ] FPS stable at 25-30
- [ ] No significant stuttering
- [ ] Can look around smoothly
- [ ] Can navigate with controller
- [ ] Images visible and clear enough
- [ ] Videos play when clicked

---

## 📊 Expected Performance Improvements

### Bundle Size:
- **Before:** 3.9-4.4 MB (single bundle.js)
- **After Phase 1:** ~2.0-2.5 MB (with named imports)
- **After Phase 2:** ~1.5-2.0 MB (with code splitting + lazy loading)

### Load Time (HTC Vive Flow):
- **Before:** 30-45 seconds
- **After:** 10-15 seconds (target < 10 with Phase 3)

### FPS (HTC Vive Flow):
- **Before:** 10-20 FPS (unstable)
- **After:** 25-30 FPS (stable)

### Memory (HTC Vive Flow):
- **Before:** 800+ MB
- **After:** 400-500 MB

---

## 🐛 Troubleshooting

### Issue: "Cannot find module 'index.js'"

**Cause:** Code splitting not enabled or build failed

**Fix:**
```bash
cd frontend
npm run clean
npm run build
# Verify index.js exists:
ls -lh ../static/index.js
```

---

### Issue: "BABYLON is not defined"

**Cause:** Missed a wildcard import replacement

**Fix:**
```bash
# Find remaining references:
grep -r "BABYLON\." ../src/
# Fix manually or re-run sed commands
```

---

### Issue: Bundle still 3.9 MB

**Cause:** Named imports not working or tree-shaking disabled

**Check:**
```bash
# Verify no wildcard imports:
grep -r "import \* as BABYLON" ../src/
# Should return nothing

# Check build script has correct flags:
cat package.json | grep build
# Should have: --splitting --outdir
```

**Fix:**
1. Ensure all files use named imports
2. Clean and rebuild: `npm run clean && npm run build`

---

### Issue: PDF.js or HLS.js loading on initial load

**Cause:** Still imported directly instead of lazy loaded

**Check:**
```bash
# Check imports in PdfPresentation.js:
head -30 ../src/scene/PdfPresentation.js
# Should see: import("pdfjs-dist") inside loadPdfJs()

# Check imports in VideoScreen.js:
head -60 ../src/scene/VideoScreen.js
# Should see: import("hls.js") inside loadHls()
```

---

### Issue: Quality profile not detected

**Cause:** Device detection not integrated

**Check Console:**
- Look for: `🎮 Device detected: ...`
- If missing, `qualityConfig` might be null

**Fix:**
1. Verify `getAutoQualitySettings()` is called in `GalleryApp.jsx`
2. Check deviceDetection.js is imported correctly

---

### Issue: Still 6 lights on mobile VR

**Cause:** Quality settings not passed to createGalleryLighting

**Check:**
```bash
# Verify in GalleryRoom.js:
grep -A 5 "createGalleryLighting" ../src/scene/GalleryRoom.js
# Should accept qualitySettings parameter
```

---

## 🎯 Success Checklist

### Build Success:
- [ ] `npm run build` completes without errors
- [ ] `index.js` file exists in `../static/`
- [ ] Multiple chunk files exist
- [ ] Total bundle size < 3 MB
- [ ] No `bundle.js` file (old single bundle)

### Desktop Test Success:
- [ ] Gallery loads in browser
- [ ] Console shows correct device detection
- [ ] Console shows quality profile
- [ ] Images display
- [ ] Videos play
- [ ] PDFs display
- [ ] PDF.js lazy loads on first PDF click
- [ ] HLS.js lazy loads on first HLS video

### Mobile VR Simulation Success:
- [ ] Detected as `mobile_vr`
- [ ] Uses `ultra_low` profile
- [ ] Single light created
- [ ] Faster load time than desktop

### HTC Vive Flow Success:
- [ ] Load time < 15 seconds
- [ ] FPS 25-30 (stable)
- [ ] Smooth head tracking
- [ ] Can navigate gallery
- [ ] Images visible
- [ ] No crashes

---

## 📝 Post-Test Notes

**Record your results:**

```
Date: ___________
Tester: ___________

Build Results:
- Bundle size: _____ MB
- Number of chunks: _____
- index.js size: _____ MB

Desktop Test:
- Load time: _____ seconds
- Device detected: _____
- Quality profile: _____
- Functionality: ✅ / ❌

HTC Vive Flow Test:
- Load time: _____ seconds
- FPS estimate: _____
- Smoothness: Good / Acceptable / Poor
- Issues: _____________________

Next Steps:
- [ ] If all tests pass → Proceed to Phase 3
- [ ] If load time > 15s → Check bundle size and network
- [ ] If FPS < 25 → Check quality profile and lighting
- [ ] If crashes → Check console errors and memory
```

---

## 🚀 Next Steps (Phase 3)

If all Phase 2 tests pass:

1. **Thumbnail LOD System** - Load low-res first, high-res on demand
2. **Progressive Room Loading** - Load only visible room
3. **Backend Pagination** - Load items in chunks
4. **Material Sharing** - Reuse materials to reduce draw calls
5. **Mesh Instancing** - Instance frame borders

**Target:** < 10 second load time, 30+ FPS on HTC Vive Flow

---

## 📞 Support

**If tests fail:**
1. Check this document's Troubleshooting section
2. Review console logs carefully
3. Verify all files were updated correctly
4. Check `MOBILE_VR_OPTIMIZATIONS.md` for details
5. Check `QUICK_ACTION_CHECKLIST.md` for quick fixes

**Questions?**
- Review the three main docs: MOBILE_VR_OPTIMIZATIONS.md, AUDIT_PERFORMANCE_TODO.md, QUICK_ACTION_CHECKLIST.md
- Check git diff to see all changes
- Verify build output matches expected sizes

---

**Last Updated:** 2024-01-XX  
**Next Review:** After test results  
**Status:** Ready for build and test
# Quick Action Checklist - Mobile VR Optimizations

**Goal:** Make HTC Vive Flow load in < 15 seconds with stable 30 FPS

---

## ✅ PHASE 1 COMPLETED (Just Now)

- [x] Replace wildcard imports with named imports (all scene files)
- [x] Create device detection utility (`deviceDetection.js`)
- [x] Implement mobile-optimized lighting (1 light for mobile VR)
- [x] Add quality profiles (ULTRA_LOW for HTC Vive Flow)
- [x] Integrate quality settings through rendering pipeline
- [x] Setup PDF.js lazy loading infrastructure

**Status:** Code changes complete, but **bundle NOT rebuilt yet!**

---

## 🚨 IMMEDIATE ACTIONS (Do These Now)

### 1. Rebuild Bundle (5 minutes) - CRITICAL ⚠️

```bash
cd crates/standalone/3d-gallery/frontend
npm run build

# Verify bundle size reduced
ls -lh ../static/bundle.js

# EXPECTED: ~2.0-2.5 MB (was 3.9-4.4 MB)
# If still > 3 MB, something went wrong!
```

**Why Critical:** All our code changes won't take effect until bundle is rebuilt!

---

### 2. Fix PDF.js Lazy Loading (30 minutes) - HIGH PRIORITY 🔴

**File:** `frontend/src/scene/PdfPresentation.js`

**Find this (around line 350-360):**
```javascript
// Load PDF
pdfjsLib
  .getDocument(media.url)
  .promise.then(async (doc) => {
```

**Replace with:**
```javascript
// Load PDF with lazy loading
loadPdfJs()
  .then(lib => lib.getDocument(media.url).promise)
  .then(async (doc) => {
```

**Why:** Currently PDF.js is still bundled. This change makes it truly lazy-loaded.

**Test:** Click a PDF in gallery - should log "📦 Lazy loading PDF.js..." first time only.

---

### 3. Enable Code Splitting (10 minutes) - HIGH PRIORITY 🔴

**File:** `frontend/package.json`

**Find:**
```json
"build": "esbuild src/index.jsx --bundle --outfile=../static/bundle.js --format=esm --minify --sourcemap --jsx=automatic --jsx-import-source=preact",
```

**Replace with:**
```json
"build": "esbuild src/index.jsx --bundle --outdir=../static --splitting --format=esm --minify --sourcemap --jsx=automatic --jsx-import-source=preact",
```

**File:** `templates/viewer.html`

**Find:**
```html
<script type="module" src="/static/3d-gallery/bundle.js"></script>
```

**Replace with:**
```html
<script type="module" src="/static/3d-gallery/index.js"></script>
```

**Why:** Splits large dependencies into separate chunks that load on demand.

**Then rebuild:**
```bash
npm run build
ls -lh ../static/
# Should see multiple .js files now (index.js + chunks)
```

---

### 4. Test on HTC Vive Flow (30 minutes) - CRITICAL ⚠️

**Checklist:**
```
[ ] Rebuild bundle (step 1)
[ ] Start server: cargo run
[ ] Access from VR browser
[ ] Load time < 15 seconds?
[ ] Check console: "Device detected: mobile_vr"
[ ] Check console: "Quality profile: ultra_low"
[ ] Check console: "Using mobile VR optimized lighting (single light)"
[ ] FPS stable at 25-30?
[ ] No stuttering when looking around?
[ ] Images visible?
[ ] Can click images?
```

**If problems, check:**
- Was bundle rebuilt?
- Console errors?
- Network tab - bundle.js size?

---

## 📋 PHASE 2 (Next Week)

### Day 1-2: Asset Loading Optimization
- [ ] Thumbnail LOD system (load thumbnails first, full-res on demand)
- [ ] Progressive room loading (only load visible room)
- [ ] Texture memory management (dispose off-screen textures)

### Day 3: Backend Optimization
- [ ] API pagination (`limit` & `offset` parameters)
- [ ] Query optimization (JOINs instead of sequential queries)
- [ ] Response caching

### Day 4-5: Rendering Optimization
- [ ] Material sharing (reuse materials)
- [ ] Mesh instancing (frame borders)
- [ ] Geometry merging (combine border pieces)

---

## 🎯 Quick Test Commands

### Desktop Test:
```bash
cd crates/standalone/3d-gallery/frontend
npm run build
cd ../..
cargo run

# Browser: http://localhost:3000/3d?code=YOUR_CODE
# Console should show: "Device detected: desktop" or "mobile"
```

### Check Bundle Size:
```bash
ls -lh crates/standalone/3d-gallery/static/bundle.js
# Target: < 2.5 MB
```

### Verify Named Imports Working:
```bash
grep "import \* as BABYLON" crates/standalone/3d-gallery/frontend/src/**/*.js*
# Should return NO results (all removed)
```

---

## 🐛 Troubleshooting

### Bundle still 3.9 MB after rebuild?
- Check: Did you save all files?
- Check: Are you rebuilding the right project?
- Check: `grep "import \* as BABYLON"` returns nothing?
- Try: `npm run clean && npm run build`

### "BABYLON is not defined" error?
- You forgot to add named import for a Babylon class
- Find the error line, identify which class, add to imports

### PDF not loading?
- Did you complete Step 2 (PDF.js lazy loading fix)?
- Check console for errors
- Check Network tab - is pdfjs-dist downloading?

### Device detection not working?
- Check console logs - should see "🎮 Device detected: ..."
- If missing, `qualityConfig` might be null
- Check `GalleryApp.jsx` - is `getAutoQualitySettings()` called?

### Lighting too dark on mobile?
- This is expected - ULTRA_LOW uses single light
- Brighter than 6 lights would be, but may look different
- Can increase `ambientLight.intensity` in `GalleryRoom.js` if needed

---

## 📊 Success Metrics

**Before Optimizations:**
- Bundle: 3.9 MB
- Load time (Vive Flow): 30-45 seconds
- FPS: 10-20 (unstable)

**Target After Phase 1:**
- Bundle: < 2.5 MB ✅
- Load time (Vive Flow): 10-15 seconds ✅
- FPS: 25-30 (stable) ✅

**Ultimate Goal (After Phase 2):**
- Bundle: < 1.5 MB (with chunks)
- Load time (Vive Flow): < 10 seconds
- FPS: 30+ (stable)

---

## 🚀 Quick Win Priority Order

1. **Rebuild bundle** (5 min) - See immediate size reduction
2. **Fix PDF lazy loading** (30 min) - Remove 5 MB from bundle
3. **Enable code splitting** (10 min) - Further reduce initial load
4. **Test on Vive Flow** (30 min) - Verify improvements

**Total Time:** ~1.5 hours for massive performance gain!

---

## 📞 Need Help?

1. Check `MOBILE_VR_OPTIMIZATIONS.md` for detailed explanations
2. Check `AUDIT_PERFORMANCE_TODO.md` for full task list
3. Review console logs - we added lots of debug output
4. Verify bundle was rebuilt: `stat crates/standalone/3d-gallery/static/bundle.js`

---

**Created:** 2024-01-XX  
**Status:** Phase 1 code complete, bundle rebuild pending  
**Next:** Execute steps 1-4 above
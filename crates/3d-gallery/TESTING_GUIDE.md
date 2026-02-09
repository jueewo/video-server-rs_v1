# 3D Gallery - Testing Guide 🧪

**Quick guide to test the 3D Gallery Phase 1 implementation**

---

## ✅ Pre-Flight Check

Before testing, verify these files exist:

```bash
# Check backend files
ls crates/3d-gallery/src/lib.rs
ls crates/3d-gallery/templates/viewer.html

# Check frontend bundle
ls crates/3d-gallery/static/bundle.js

# Should show ~3.9MB
du -h crates/3d-gallery/static/bundle.js
```

If `bundle.js` is missing:
```bash
cd crates/3d-gallery/frontend
npm install
npm run build
cd ../../..
```

---

## 🚀 Test 1: Start the Server

```bash
# From project root
cargo run
```

**Expected output:**
```
🚀 Initializing Modular Media Server...
📊 Database: sqlite:media.db
🔐 OIDC Configuration: ...
✅ Server starting on http://0.0.0.0:3000
```

If build fails, check `Cargo.toml` includes `gallery3d` dependency.

---

## 🎫 Test 2: Create an Access Code

You need a valid access code to test the 3D gallery.

### Option A: Via Demo Page

1. Visit: `http://localhost:3000/demo`
2. Enter an access code name (e.g., "test123")
3. Click "Generate Access Code"
4. Copy the generated code

### Option B: Via SQL (Quick)

```bash
# Open SQLite database
sqlite3 media.db

# Create a test access code
INSERT INTO access_codes (code, name, expires_at, created_at)
VALUES ('testgallery', 'Test Gallery', datetime('now', '+7 days'), datetime('now'));

# Exit
.quit
```

---

## 🎨 Test 3: Access the 3D Gallery

Visit one of these URLs (replace `YOUR_CODE` with your access code):

```
http://localhost:3000/3d?code=YOUR_CODE
http://localhost:3000/digital-twin?code=YOUR_CODE
```

**Example:**
```
http://localhost:3000/3d?code=testgallery
```

---

## ✨ Test 4: Verify Scene Loads

When the page loads, you should see:

### Loading Phase (1-2 seconds)
- ✅ Purple gradient background
- ✅ White spinning loader
- ✅ "Loading 3D Gallery..." text

### 3D Scene Phase
- ✅ Black canvas fills screen
- ✅ Blue rotating cube in center
- ✅ Gray ground plane below
- ✅ Dark skybox around scene
- ✅ Smooth lighting

### UI Elements
- ✅ Help overlay (bottom-left corner)
- ✅ Controls listed (Mouse, WASD, Scroll, etc.)
- ✅ "Got it!" button to dismiss help

---

## 🎮 Test 5: Camera Controls

### Mouse Controls
1. **Click and drag** on canvas
   - Expected: Camera rotates around the cube
   - The cube should stay in center while view rotates

2. **Scroll wheel**
   - Expected: Camera zooms in/out
   - Should stop at min distance (2 units) and max distance (50 units)

3. **Click "Got it!"** on help overlay
   - Expected: Help overlay disappears

### Verify Smooth Performance
- Cube should rotate smoothly (60 FPS target)
- No stuttering or lag
- Controls should feel responsive

---

## 🔍 Test 6: Browser Console Check

Open browser DevTools (F12) and check Console:

**Expected messages:**
```
Initializing Babylon.js scene...
Babylon.js scene initialized successfully!
Gallery data: {items: [], scene: "classic", ...}
```

**Should NOT see:**
- ❌ React is not defined
- ❌ JSX syntax errors
- ❌ Module not found errors
- ❌ WebGL errors

---

## 🚫 Test 7: Error Handling

### Test Invalid Code
```
http://localhost:3000/3d?code=invalid999
```

**Expected:**
- White error card appears
- ⚠️ Warning icon
- Title: "Failed to Load Gallery"
- Message: "Invalid or expired access code"
- No 3D scene

### Test Missing Code
```
http://localhost:3000/3d
```

**Expected:**
- White error card appears
- ⚠️ Warning icon
- Title: "Access Code Required"
- Message: "No access code provided..."

---

## 📱 Test 8: Browser Compatibility

Test on multiple browsers:

### Desktop Browsers
- [ ] Chrome/Chromium (latest)
- [ ] Firefox (latest)
- [ ] Safari (macOS)
- [ ] Edge (Windows)

### Mobile Browsers (if available)
- [ ] Safari iOS
- [ ] Chrome Android

**For each browser, verify:**
- Scene loads and renders
- Controls work
- No console errors
- Performance is acceptable

---

## 🐛 Troubleshooting

### Issue: "React is not defined" error

**Solution:**
```bash
cd crates/3d-gallery/frontend
npm run build
```

Verify `package.json` has JSX config:
```json
"--jsx=automatic --jsx-import-source=preact"
```

### Issue: Black screen, no cube

**Check:**
1. Browser console for WebGL errors
2. WebGL support: Visit https://get.webgl.org/
3. GPU acceleration enabled in browser settings

**Try:**
```javascript
// Add to GalleryApp.jsx after scene creation:
scene.debugLayer.show();
```

### Issue: Bundle.js 404 Not Found

**Check:**
```bash
ls -lh crates/3d-gallery/static/bundle.js
```

If missing:
```bash
cd crates/3d-gallery/frontend
npm install
npm run build
```

### Issue: Access code validation fails

**Check database:**
```bash
sqlite3 media.db "SELECT * FROM access_codes;"
```

**Check backend logs** for error messages.

**Test API directly:**
```bash
curl "http://localhost:3000/api/3d/gallery?code=testgallery"
```

Should return JSON with `items`, `scene`, `permissions`.

### Issue: Performance problems (low FPS)

**Check:**
- GPU acceleration enabled
- No other heavy apps running
- Browser hardware acceleration on

**Add FPS counter:**
```javascript
// In GalleryApp.jsx
scene.debugLayer.show();
// Click "Inspector" -> "Stats" tab
```

---

## 📊 Success Checklist

Mark each item when verified:

### Backend
- [ ] Server starts without errors
- [ ] `/3d` route accessible
- [ ] `/api/3d/gallery` endpoint works
- [ ] Access code validation working
- [ ] Static files served from `/static/3d-gallery/`

### Frontend
- [ ] Bundle.js loads (check Network tab)
- [ ] No console errors
- [ ] Loading screen appears and hides
- [ ] 3D scene initializes
- [ ] Babylon.js engine running

### 3D Scene
- [ ] Cube visible and rotating
- [ ] Ground plane visible
- [ ] Skybox present
- [ ] Lighting looks good
- [ ] Camera controls work (drag, zoom)
- [ ] Smooth performance (60 FPS)

### UI/UX
- [ ] Help overlay shows
- [ ] "Got it!" button works
- [ ] Error states work (invalid code)
- [ ] Mobile responsive (if testing mobile)

### Integration
- [ ] Access code validation working
- [ ] API returns gallery data
- [ ] Frontend consumes API data
- [ ] No integration errors

---

## 🎉 All Tests Pass?

**Congratulations!** Phase 1 is working correctly!

**Next steps:**
1. Commit your changes
2. Push to `feature/3d-gallery` branch
3. Move on to Phase 2 (Gallery Room)

See `IMPLEMENTATION_PLAN.md` for Phase 2 details.

---

## 📞 Need Help?

If tests fail:

1. **Check logs** - Backend and browser console
2. **Verify files** - Use checklist above
3. **Rebuild** - Clean build both frontend and backend
4. **Review docs** - `README.md`, `PHASE1_COMPLETE.md`

**Common fixes:**
- Rebuild frontend: `npm run build`
- Restart server: `cargo run`
- Clear browser cache
- Check WebGL support

---

**Happy Testing!** 🚀✨
# 3D Gallery - Quick Reference Card 📋

**Essential commands and URLs for daily development**

---

## 🚀 Quick Start (First Time)

```bash
# 1. Install frontend dependencies
cd crates/3d-gallery/frontend
npm install

# 2. Build frontend bundle
npm run build

# 3. Start server
cd ../../..
cargo run

# 4. Open browser
# http://localhost:3000/3d?code=YOUR_CODE
```

---

## 📦 Build Commands

### Frontend

```bash
cd crates/3d-gallery/frontend

# Production build (minified)
npm run build

# Development (watch mode)
npm run dev

# Clean artifacts
npm run clean
```

### Backend

```bash
# From project root

# Build
cargo build

# Run
cargo run

# Watch mode (if cargo-watch installed)
cargo watch -x run

# Clean
cargo clean
```

---

## 🌐 URLs

### Main Routes
```
http://localhost:3000/3d?code=YOUR_CODE
http://localhost:3000/digital-twin?code=YOUR_CODE
```

### API Endpoint
```
http://localhost:3000/api/3d/gallery?code=YOUR_CODE
```

### Static Files
```
http://localhost:3000/static/3d-gallery/bundle.js
```

---

## 🎫 Create Test Access Code

### Via SQL (Quick)
```bash
sqlite3 media.db
```
```sql
INSERT INTO access_codes (code, name, expires_at, created_at)
VALUES ('testgallery', 'Test', datetime('now', '+7 days'), datetime('now'));
```
```bash
.quit
```

### Via Demo Page
```
http://localhost:3000/demo
```

---

## 🐛 Debug Commands

### Check Bundle Exists
```bash
ls -lh crates/3d-gallery/static/bundle.js
# Should show ~3.9MB
```

### Check Database
```bash
sqlite3 media.db "SELECT * FROM access_codes;"
```

### Test API
```bash
curl "http://localhost:3000/api/3d/gallery?code=testgallery"
```

### Check WebGL Support
```
https://get.webgl.org/
```

---

## 🔧 Common Fixes

### "React is not defined"
```bash
cd crates/3d-gallery/frontend
npm run build
```

### Bundle 404
```bash
cd crates/3d-gallery/frontend
npm install
npm run build
```

### Rust Build Error
```bash
cargo clean
cargo build
```

### Black Screen
- Check browser console
- Verify WebGL support
- Check bundle loaded (Network tab)

---

## 📁 Key Files

### Backend
```
src/lib.rs           # Main module
src/routes.rs        # HTTP routes
src/api.rs           # API endpoints
templates/viewer.html # HTML template
```

### Frontend
```
frontend/src/index.jsx       # Entry point
frontend/src/GalleryApp.jsx  # Main component
frontend/src/api/galleryApi.js # API client
```

### Config
```
frontend/package.json  # Dependencies
Cargo.toml            # Rust config
```

---

## 🎮 Browser Controls

- **Mouse drag** - Rotate camera
- **Scroll wheel** - Zoom in/out
- **ESC** - Exit pointer lock

---

## 📊 Bundle Info

- **Size:** 3.9 MB minified
- **Location:** `crates/3d-gallery/static/bundle.js`
- **Format:** ESM
- **Sourcemap:** `bundle.js.map` (16.5 MB)

---

## 🧪 Quick Test

```bash
# 1. Start server
cargo run

# 2. Visit (replace YOUR_CODE)
# http://localhost:3000/3d?code=testgallery

# 3. Expect:
# - Loading screen
# - Rotating blue cube
# - Help overlay (bottom-left)
# - 60 FPS
```

---

## 📚 Documentation

```
README.md                 # Overview
IMPLEMENTATION_PLAN.md    # Roadmap
PHASE1_COMPLETE.md        # Phase 1 guide
TESTING_GUIDE.md          # How to test
COMPLETION_SUMMARY.md     # What's done
frontend/README.md        # Frontend docs
```

---

## 💡 Tips

- Keep `npm run dev` running while developing frontend
- Check browser console for errors
- Use Babylon.js inspector: `scene.debugLayer.show()`
- WebGL 2.0 required
- Works best on desktop browsers

---

## 🆘 Help

**If stuck:**
1. Read `TESTING_GUIDE.md`
2. Check browser console
3. Verify bundle exists
4. Rebuild frontend
5. Restart server

---

**Phase 1 Complete!** ✅  
**Next:** Phase 2 - Gallery Room

**Happy Coding!** 🚀
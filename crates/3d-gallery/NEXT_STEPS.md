# 3D Gallery - Next Steps (Phase 1 Implementation)

**Current Status:** Planning complete, ready to start implementation  
**Branch:** `feature/3d-gallery`  
**Phase:** Phase 1 - Core Infrastructure (Week 1)

---

## 🎯 Immediate Next Steps (Start Here!)

### Step 1: Backend Foundation (Day 1-2)

#### 1.1 Create Basic Module Structure

```bash
# Create source files
touch crates/3d-gallery/src/lib.rs
touch crates/3d-gallery/src/routes.rs
touch crates/3d-gallery/src/api.rs
touch crates/3d-gallery/src/models.rs

# Create templates directory
mkdir -p crates/3d-gallery/templates
touch crates/3d-gallery/templates/viewer.html
```

**Files to create:**
- [ ] `src/lib.rs` - Main module, export router
- [ ] `src/routes.rs` - HTTP routes (`/3d`, `/digital-twin`)
- [ ] `src/api.rs` - JSON API endpoints
- [ ] `src/models.rs` - Data structures
- [ ] `templates/viewer.html` - Main viewer page

#### 1.2 Implement Basic Router

**`src/lib.rs`:**
```rust
use axum::{routing::get, Router};

pub mod api;
pub mod models;
pub mod routes;

pub fn router() -> Router {
    Router::new()
        .route("/3d", get(routes::viewer_page))
        .route("/api/3d/gallery", get(api::get_gallery_data))
}
```

#### 1.3 Implement Access Code Route

**`src/routes.rs`:**
```rust
use axum::{
    extract::Query,
    response::Html,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GalleryQuery {
    code: String,  // Required access code
}

pub async fn viewer_page(
    Query(query): Query<GalleryQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    // Validate access code
    // Render template
    // Return HTML
}
```

#### 1.4 Register in Main Router

**In `video-server-rs_v1/src/main.rs`:**
```rust
use threeD_gallery;  // Note: crate name uses underscore

let app = Router::new()
    .merge(video_manager::router())
    .merge(image_manager::router())
    .merge(threeD_gallery::router())  // Add this!
    // ...
```

#### 1.5 Add to Workspace

**In `video-server-rs_v1/Cargo.toml`:**
```toml
[workspace]
members = [
    # ...
    "crates/media-mcp",
    "crates/3d-gallery",  # Add this!
    # ...
]
```

---

### Step 2: Frontend Setup (Day 2-3)

#### 2.1 Create Frontend Directory Structure

```bash
# Create frontend directories
mkdir -p crates/3d-gallery/frontend/src/scene
mkdir -p crates/3d-gallery/frontend/src/components
mkdir -p crates/3d-gallery/frontend/src/api
mkdir -p crates/3d-gallery/static

# Create frontend files
touch crates/3d-gallery/frontend/package.json
touch crates/3d-gallery/frontend/src/index.jsx
touch crates/3d-gallery/frontend/src/GalleryApp.jsx
touch crates/3d-gallery/frontend/src/scene/SceneManager.js
touch crates/3d-gallery/frontend/src/api/galleryApi.js
```

#### 2.2 Setup package.json

**`frontend/package.json`:**
```json
{
  "name": "3d-gallery-frontend",
  "version": "0.1.0",
  "scripts": {
    "build": "esbuild src/index.jsx --bundle --outfile=../static/bundle.js --format=esm --minify",
    "dev": "esbuild src/index.jsx --bundle --outfile=../static/bundle.js --format=esm --watch",
    "clean": "rm -f ../static/bundle.js"
  },
  "dependencies": {
    "preact": "^10.19.0",
    "@babylonjs/core": "^7.0.0",
    "@babylonjs/loaders": "^7.0.0"
  },
  "devDependencies": {
    "esbuild": "^0.19.0"
  }
}
```

#### 2.3 Install Dependencies

```bash
cd crates/3d-gallery/frontend
npm install
```

#### 2.4 Create Minimal Preact Component

**`frontend/src/index.jsx`:**
```jsx
import { h, render } from 'preact';
import GalleryApp from './GalleryApp';

// Get access code from URL
const params = new URLSearchParams(window.location.search);
const accessCode = params.get('code');

// Render app
render(
  <GalleryApp code={accessCode} />,
  document.getElementById('gallery-root')
);
```

#### 2.5 Create Basic Babylon.js Scene

**`frontend/src/GalleryApp.jsx`:**
```jsx
import { h } from 'preact';
import { useEffect, useRef } from 'preact/hooks';
import * as BABYLON from '@babylonjs/core';

export default function GalleryApp({ code }) {
  const canvasRef = useRef(null);

  useEffect(() => {
    if (!canvasRef.current) return;

    // Create engine
    const engine = new BABYLON.Engine(canvasRef.current);
    const scene = new BABYLON.Scene(engine);

    // Camera
    const camera = new BABYLON.ArcRotateCamera(
      "camera",
      Math.PI / 2,
      Math.PI / 2,
      10,
      BABYLON.Vector3.Zero(),
      scene
    );
    camera.attachControl(canvasRef.current, true);

    // Light
    const light = new BABYLON.HemisphericLight(
      "light",
      new BABYLON.Vector3(0, 1, 0),
      scene
    );

    // Test cube
    const box = BABYLON.MeshBuilder.CreateBox("box", { size: 2 }, scene);

    // Render loop
    engine.runRenderLoop(() => scene.render());

    // Cleanup
    return () => {
      engine.dispose();
    };
  }, []);

  return (
    <div style={{ width: '100%', height: '100vh' }}>
      <canvas ref={canvasRef} style={{ width: '100%', height: '100%' }} />
    </div>
  );
}
```

#### 2.6 Build Frontend

```bash
cd crates/3d-gallery/frontend
npm run build
```

---

### Step 3: Template & Testing (Day 3-4)

#### 3.1 Create Viewer Template

**`templates/viewer.html`:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>3D Gallery</title>
    <style>
        body, html {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
        }
    </style>
</head>
<body>
    <div id="gallery-root"></div>
    <script type="module" src="/static/3d-gallery/bundle.js"></script>
</body>
</html>
```

#### 3.2 Serve Static Files

**In `src/lib.rs`:**
```rust
use tower_http::services::ServeDir;

pub fn router() -> Router {
    Router::new()
        .route("/3d", get(routes::viewer_page))
        .route("/api/3d/gallery", get(api::get_gallery_data))
        .nest_service(
            "/static/3d-gallery",
            ServeDir::new("crates/3d-gallery/static")
        )
}
```

#### 3.3 Test Basic Setup

```bash
# Build everything
cargo build

# Run server
cargo run

# Open browser
# http://localhost:3000/3d?code=test123
```

**Expected Result:**
- Page loads without errors
- Black canvas appears
- A rotating cube is visible (test scene)

---

### Step 4: Access Code Integration (Day 4-5)

#### 4.1 Implement Code Validation

**`src/api.rs`:**
```rust
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GalleryQuery {
    code: String,
}

#[derive(Serialize)]
pub struct GalleryResponse {
    items: Vec<MediaItem3D>,
    scene: String,
    permissions: Permissions,
}

pub async fn get_gallery_data(
    Query(query): Query<GalleryQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<GalleryResponse>, (StatusCode, String)> {
    // 1. Validate access code
    let access_code = validate_access_code(&query.code, &state).await?;
    
    // 2. Get permissions
    let permissions = get_code_permissions(&access_code, &state).await?;
    
    // 3. Fetch media
    let media = fetch_media_for_code(&permissions, &state).await?;
    
    // 4. Transform to 3D format
    let items = transform_to_3d(media);
    
    Ok(Json(GalleryResponse {
        items,
        scene: "classic".to_string(),
        permissions,
    }))
}
```

#### 4.2 Add Error Handling

**Error states to handle:**
- [ ] Invalid access code
- [ ] Expired access code
- [ ] Revoked access code
- [ ] No media found
- [ ] Database errors

#### 4.3 Test with Real Access Code

```bash
# 1. Create access code via main UI
# 2. Note the code
# 3. Test: http://localhost:3000/3d?code=YOUR_CODE
```

---

## 📋 Phase 1 Checklist

### Backend Tasks
- [ ] Create module structure (lib.rs, routes.rs, api.rs, models.rs)
- [ ] Implement basic router
- [ ] Add access code validation
- [ ] Create viewer route (`/3d`)
- [ ] Create API endpoint (`/api/3d/gallery`)
- [ ] Integrate with access-codes crate
- [ ] Handle error states (invalid/expired codes)
- [ ] Register router in main.rs
- [ ] Add to workspace Cargo.toml
- [ ] Serve static files

### Frontend Tasks
- [ ] Create frontend directory structure
- [ ] Setup package.json with dependencies
- [ ] Install npm packages (Preact, Babylon.js)
- [ ] Create basic Preact component
- [ ] Initialize Babylon.js engine
- [ ] Create test scene (cube)
- [ ] Setup build script (esbuild)
- [ ] Build frontend bundle
- [ ] Test in browser

### Integration Tasks
- [ ] Create viewer template (HTML)
- [ ] Link frontend bundle in template
- [ ] Test access code validation
- [ ] Verify static file serving
- [ ] Test error pages
- [ ] Verify camera controls work
- [ ] Check mobile compatibility

### Testing Tasks
- [ ] Unit tests for access code validation
- [ ] Integration test for API endpoint
- [ ] Browser test (Chrome, Firefox, Safari)
- [ ] Mobile test (iOS, Android)
- [ ] Performance check (60fps?)

### Documentation Tasks
- [ ] Update README with build instructions
- [ ] Document API endpoints
- [ ] Add troubleshooting tips
- [ ] Update IMPLEMENTATION_PLAN progress

---

## 🎯 Success Criteria for Phase 1

**Minimum Viable Prototype:**
- ✅ User can visit `/3d?code=xyz`
- ✅ Access code is validated
- ✅ Invalid codes show error page
- ✅ Valid codes load 3D viewer
- ✅ Babylon.js scene renders (test cube visible)
- ✅ Camera controls work (rotate, zoom)
- ✅ No console errors
- ✅ Works on desktop browsers

**Nice to Have:**
- 🎁 Loading indicator
- 🎁 Help overlay with controls
- 🎁 Mobile touch controls
- 🎁 FPS counter (debug mode)

---

## 🚀 Quick Start Commands

```bash
# Switch to feature branch
git checkout feature/3d-gallery

# Create directory structure
mkdir -p crates/3d-gallery/src
mkdir -p crates/3d-gallery/templates
mkdir -p crates/3d-gallery/frontend/src/scene
mkdir -p crates/3d-gallery/frontend/src/components
mkdir -p crates/3d-gallery/frontend/src/api
mkdir -p crates/3d-gallery/static

# Create placeholder files
touch crates/3d-gallery/src/{lib.rs,routes.rs,api.rs,models.rs}
touch crates/3d-gallery/templates/viewer.html
touch crates/3d-gallery/frontend/package.json
touch crates/3d-gallery/frontend/src/index.jsx

# Setup frontend
cd crates/3d-gallery/frontend
npm install
npm run build
cd ../../..

# Build and test
cargo build
cargo run

# Visit in browser
# http://localhost:3000/3d?code=test123
```

---

## 📚 Reference Documentation

- **Planning:** `IMPLEMENTATION_PLAN.md` - Full roadmap
- **Access Model:** `ACCESS_MODEL.md` - Security and access control
- **Overview:** `README.md` - Project overview
- **Babylon.js Docs:** https://doc.babylonjs.com/
- **Preact Docs:** https://preactjs.com/

---

## ⏱️ Time Estimates

| Task | Estimated Time |
|------|----------------|
| Backend structure | 2-4 hours |
| Access code integration | 3-5 hours |
| Frontend setup | 2-3 hours |
| Basic Babylon.js scene | 2-4 hours |
| Template & static serving | 1-2 hours |
| Testing & debugging | 3-5 hours |
| **Total Phase 1** | **13-23 hours** (~2-3 days) |

---

## 🎉 After Phase 1

Once you have a working prototype with a test cube:

**Phase 2:** Replace cube with actual gallery room
**Phase 3:** Load real images as textures
**Phase 4:** Add video screens
**Phase 5:** Multiple scenes

See `IMPLEMENTATION_PLAN.md` for complete roadmap.

---

## 🆘 Need Help?

**Common Issues:**
- Babylon.js not loading → Check bundle.js path
- Black screen → Check browser console for WebGL errors
- Access denied → Verify access code in database
- Build errors → Check Cargo.toml dependencies

**Resources:**
- Babylon.js Playground: https://playground.babylonjs.com/
- WebGL support check: https://get.webgl.org/
- Project README: `crates/3d-gallery/README.md`

---

**Status:** Ready to start Phase 1 implementation!  
**Next Action:** Create `src/lib.rs` and basic router  
**Branch:** `feature/3d-gallery`  
**Let's build something amazing!** 🚀✨
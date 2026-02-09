# 3D Gallery Implementation Plan

## Overview

A 3D virtual gallery using Preact and Babylon.js to display images and videos from the media server in an immersive 3D environment. Users can explore a virtual space where media items are displayed on walls, screens, and interactive surfaces.

---

## Goals

1. **Immersive Media Viewing** - Browse media library in a 3D environment
2. **Integration with Existing System** - Use media server's authentication, permissions, and APIs
3. **Performance** - Smooth 60fps rendering with lazy loading
4. **Interactivity** - Click, navigate, and interact with media in 3D space
5. **Extensibility** - Foundation for future VR/AR features

---

## Phase 1: Core Infrastructure (Week 1)

### Backend (Rust)

**Deliverables:**
- [ ] Create `crates/3d-gallery` module
- [ ] Basic Cargo.toml with dependencies
- [ ] Module registration in main router
- [ ] API endpoint: `GET /api/3d/gallery` - Returns media items optimized for 3D
- [ ] API endpoint: `GET /api/3d/scenes` - Available scene layouts
- [ ] Template: `viewer.html` - Main 3D viewer page
- [ ] Static file serving for bundled JS

**API Response Structure:**
```rust
struct GalleryScene {
    id: String,
    name: String,
    environment: EnvironmentType,
    media_items: Vec<MediaItem3D>,
    camera_presets: Vec<CameraPosition>,
}

struct MediaItem3D {
    id: i32,
    media_type: MediaType, // Image, Video
    url: String,
    thumbnail_url: String,
    title: String,
    description: Option<String>,
    position: Position3D,
    rotation: Rotation3D,
    scale: f32,
}

struct Position3D {
    x: f32,
    y: f32,
    z: f32,
}
```

**Dependencies:**
```toml
[dependencies]
common = { path = "../common" }
video-manager = { path = "../video-manager" }
image-manager = { path = "../image-manager" }
access-control = { path = "../access-control" }
axum = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = "1.0"
askama = { workspace = true }
```

---

### Frontend (Preact + Babylon.js)

**Deliverables:**
- [ ] Setup `frontend/` directory structure
- [ ] package.json with Preact and Babylon.js
- [ ] Basic Babylon.js scene initialization
- [ ] Camera controls (WASD + mouse look)
- [ ] API client for fetching gallery data
- [ ] Build script (esbuild)

**Directory Structure:**
```
crates/3d-gallery/
├── Cargo.toml
├── package.json
├── src/
│   ├── lib.rs
│   ├── routes.rs
│   ├── api.rs
│   └── models.rs
├── templates/
│   └── viewer.html
├── frontend/
│   ├── package.json
│   ├── src/
│   │   ├── index.jsx
│   │   ├── GalleryApp.jsx
│   │   ├── scene/
│   │   │   ├── SceneManager.js
│   │   │   └── CameraController.js
│   │   └── api/
│   │       └── galleryApi.js
└── static/
    └── bundle.js (generated)
```

**Dependencies:**
```json
{
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

---

## Phase 2: Basic Gallery Scene (Week 2)

### Features

**Deliverables:**
- [ ] Gallery room environment (walls, floor, ceiling)
- [ ] Image frames on walls
- [ ] Basic lighting (ambient + spotlights)
- [ ] Texture loading from media server
- [ ] Click interaction on images
- [ ] Info panel overlay (title, description)
- [ ] Navigation UI (minimap, controls help)

**Scene Layout:**
```
Gallery Room:
- 4 walls (20m x 4m each)
- Wooden floor with texture
- Ceiling with skylights
- Picture frames auto-arranged on walls
- Spacing: 2-3m between frames
- Height: 1.5-2m from floor
```

**Interactions:**
- Click image → Show full info overlay
- ESC → Close overlay
- Arrow keys / WASD → Navigate
- Mouse → Look around
- Scroll → Zoom (adjust camera FOV)

---

## Phase 3: Video Integration (Week 3)

### Features

**Deliverables:**
- [ ] Video texture support in Babylon.js
- [ ] Video "screens" in 3D space
- [ ] Play/Pause controls
- [ ] Volume controls
- [ ] Video preview on hover
- [ ] Multiple video screens in scene
- [ ] HLS streaming support (optional)

**Video Screen Types:**
- Wall-mounted screens (16:9 aspect ratio)
- Cinema screen (large format)
- Multiple small monitors (grid layout)

**Controls:**
- Click screen → Toggle play/pause
- Hover → Show video scrubber
- Volume icon in UI overlay

---

## Phase 4: Multiple Scenes & Layouts (Week 4)

### Scene Types

**Deliverables:**
- [ ] API: Scene configuration system
- [ ] Scene selector UI
- [ ] Transition between scenes
- [ ] Scene 1: Classic Gallery (white walls, wood floor)
- [ ] Scene 2: Modern Gallery (concrete, minimalist)
- [ ] Scene 3: Outdoor Plaza (sky, terrain, billboards)
- [ ] Scene 4: Virtual Office (desks, screens, decorations)
- [ ] Save/Load custom layouts

**Scene Switching:**
```javascript
scenes: [
  { id: 'classic', name: 'Classic Gallery', icon: '🖼️' },
  { id: 'modern', name: 'Modern Space', icon: '🏢' },
  { id: 'outdoor', name: 'Outdoor Plaza', icon: '🌳' },
  { id: 'office', name: 'Virtual Office', icon: '💼' },
]
```

---

## Phase 5: Advanced Features (Week 5-6)

### Features

**Deliverables:**
- [ ] Dynamic lighting (day/night cycle)
- [ ] Ambient audio support
- [ ] Particle effects (dust, atmosphere)
- [ ] Post-processing effects (bloom, SSAO)
- [ ] Teleport markers for navigation
- [ ] Tag filtering in 3D UI
- [ ] Group-specific galleries
- [ ] Load more / Pagination
- [ ] Performance optimization (LOD, culling)
- [ ] Loading screen with progress

**UI Components:**
- Floating menu panel
- Tag filter buttons
- Scene selector dropdown
- Media counter
- FPS display (debug mode)

---

## Phase 6: Polish & UX (Week 7)

### Features

**Deliverables:**
- [ ] Smooth camera transitions
- [ ] Animation system (frame entrance/exit)
- [ ] Sound effects (footsteps, clicks)
- [ ] Help overlay (keyboard shortcuts)
- [ ] Mobile support (touch controls)
- [ ] Responsive layout adjustments
- [ ] Loading state improvements
- [ ] Error handling UI
- [ ] Accessibility features (keyboard-only navigation)

**Performance:**
- Target: 60fps on mid-range hardware
- Texture streaming for large images
- Lazy loading of off-screen media
- Memory management (texture cleanup)

---

## Phase 7: VR/AR Support (Future - Week 8+)

### Features (Optional)

**Deliverables:**
- [ ] WebXR API integration
- [ ] VR headset support (Quest, PSVR, etc.)
- [ ] Hand tracking / controller support
- [ ] Teleportation locomotion
- [ ] AR mode (mobile devices)
- [ ] Spatial audio
- [ ] Multi-user support (future)

**VR Features:**
- Immersive viewing mode
- Grab and inspect images
- Voice commands (with MCP integration?)
- Social viewing (multiplayer)

---

## Technical Architecture

### Data Flow

```
┌──────────────────┐
│  Media Server    │
│  (Rust/Axum)     │
│                  │
│  /api/3d/gallery │◄─────┐
│  /storage/*      │      │
└──────────────────┘      │
                          │ HTTP/JSON
┌──────────────────┐      │
│  Browser         │      │
│                  │      │
│  Preact App      │──────┘
│  ├─ API Client   │
│  └─ Babylon Scene│
│     ├─ Camera    │
│     ├─ Lights    │
│     ├─ Meshes    │
│     └─ Textures  │
└──────────────────┘
```

### Performance Considerations

**Optimization Strategies:**
1. **Texture Management**
   - Use thumbnails for distant objects
   - Load full-res on approach
   - Unload textures when out of view
   - Texture compression (basis, KTX2)

2. **Geometry**
   - Simple planes for frames (low poly)
   - LOD system for complex scenes
   - Instancing for repeated objects

3. **Rendering**
   - Frustum culling
   - Occlusion culling
   - Shadow optimization
   - Reduce draw calls

4. **Loading**
   - Progressive loading
   - Priority queue (visible first)
   - Background loading for adjacent areas

---

## Integration Points

### Authentication & Authorization
- Use existing session system
- Respect group permissions
- Only show accessible media

### Media Server APIs
- Leverage existing `/api/videos` endpoints
- Leverage existing `/api/images` endpoints
- Transform data for 3D format

### Navigation Integration
- Link from media-hub to 3D gallery
- Link from 3D gallery back to traditional views
- Breadcrumb navigation

---

## Testing Strategy

### Unit Tests
- API endpoint responses
- Scene data transformation
- Permission checks

### Integration Tests
- Full data flow (API → Frontend)
- Scene loading
- Media texture loading

### Performance Tests
- FPS benchmarks
- Memory usage monitoring
- Load testing (100+ media items)

### Browser Tests
- Chrome, Firefox, Safari
- Mobile browsers
- WebGL compatibility

---

## Documentation

**Deliverables:**
- [ ] README.md - Overview and quick start
- [ ] ARCHITECTURE.md - Technical details
- [ ] USER_GUIDE.md - How to use the 3D gallery
- [ ] API.md - API endpoint documentation
- [ ] DEVELOPMENT.md - Developer setup guide

---

## Success Metrics

### Performance Targets
- 60fps on desktop (1080p)
- 30fps on mobile (720p)
- <3s initial load time
- <500ms scene transitions

### User Experience
- Intuitive navigation (no tutorial needed)
- Smooth interactions
- Fast media loading
- No crashes or freezes

### Integration
- Works with all existing media types
- Respects all permissions
- Compatible with existing auth

---

## Future Enhancements (Post-MVP)

### Features
- [ ] Custom scene editor
- [ ] Animated transitions between images
- [ ] Slideshow mode
- [ ] Multiplayer/social viewing
- [ ] User-created exhibitions
- [ ] Audio guides
- [ ] Annotation tools
- [ ] Export scene configurations
- [ ] AI-powered auto-layout
- [ ] Integration with MCP (ask Claude to arrange gallery)

### Advanced Rendering
- [ ] Ray tracing (WebGPU)
- [ ] Photorealistic materials
- [ ] Real-time reflections
- [ ] Global illumination
- [ ] HDR rendering

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1 | Week 1 | Core infrastructure, basic scene |
| Phase 2 | Week 2 | Gallery room, image display |
| Phase 3 | Week 3 | Video integration |
| Phase 4 | Week 4 | Multiple scenes |
| Phase 5 | Week 5-6 | Advanced features, polish |
| Phase 6 | Week 7 | UX improvements |
| Phase 7 | Week 8+ | VR/AR (optional) |

**Total:** 7-8 weeks for MVP (Phases 1-6)

---

## Dependencies & Prerequisites

### Required Knowledge
- Rust/Axum (backend)
- Preact/JSX (frontend)
- Babylon.js API
- WebGL concepts
- 3D mathematics basics

### External Dependencies
- Babylon.js (3D engine)
- Preact (UI framework)
- esbuild (bundler)

### System Requirements
- WebGL 2.0 support
- Modern browser (Chrome 90+, Firefox 88+, Safari 15+)
- 2GB+ RAM
- Decent GPU (integrated graphics OK for basic scenes)

---

## Risk Assessment

### Technical Risks
- **Browser compatibility** - Mitigation: WebGL fallbacks, feature detection
- **Performance on low-end devices** - Mitigation: Quality settings, mobile optimization
- **Large texture memory usage** - Mitigation: Streaming, compression, LOD

### Integration Risks
- **API changes** - Mitigation: Versioned APIs, good documentation
- **Permission complexity** - Mitigation: Reuse existing access-control crate

### User Experience Risks
- **Learning curve** - Mitigation: Tutorial, help overlay, intuitive controls
- **Motion sickness** - Mitigation: Camera smoothing, teleport option

---

## Getting Started

### Developer Setup

```bash
# 1. Build backend
cd crates/3d-gallery
cargo build

# 2. Install frontend dependencies
npm install

# 3. Build frontend
npm run build

# 4. Run development mode
npm run dev  # Watch mode for frontend
cargo run    # From project root
```

### First Steps

1. Create basic route in `src/lib.rs`
2. Add template in `templates/viewer.html`
3. Initialize Babylon.js scene in `frontend/src/index.jsx`
4. Test with simple cube before adding media

---

**Status:** 📋 Planning Phase  
**Priority:** Medium (Enhancement Feature)  
**Estimated Effort:** 7-8 weeks  
**Dependencies:** None (standalone enhancement)
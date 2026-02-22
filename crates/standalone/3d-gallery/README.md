# 3D Gallery - Immersive Media Viewing

**An immersive 3D virtual gallery for exploring images and videos using Preact and Babylon.js**

---

## 🎨 Overview

The 3D Gallery transforms your media library into an interactive 3D experience. Walk through virtual gallery rooms where your photos hang on walls and videos play on screens, all rendered in real-time with WebGL.

**🔑 No Login Required!** Share galleries via access codes - perfect for client presentations, exhibitions, and public showcases.

### Key Features

- 🔓 **Anonymous Access** - View galleries with just an access code (no account needed)
- 🔗 **Easy Sharing** - Share a link, anyone can view
- 🖼️ **Virtual Gallery Rooms** - Explore 3D spaces with realistic lighting and materials
- 📸 **Image Walls** - Photos displayed as framed artworks on gallery walls
- 🎬 **Video Screens** - Videos play on 3D screens with full playback controls
- 🎮 **Interactive Navigation** - WASD movement, mouse look, click interactions
- 🏛️ **Multiple Scenes** - Classic gallery, modern space, outdoor plaza, office
- 📱 **Mobile Support** - Touch controls and responsive design
- ⚡ **Performance** - Optimized with texture streaming, LOD, and culling
- 🔐 **Secure** - Access codes can expire and be revoked

### Perfect For

- 📸 **Photographers** - Share portfolios with clients
- 🎨 **Artists** - Create public exhibitions
- 🏢 **Companies** - Share project galleries with stakeholders
- 🎉 **Events** - Share photos with attendees
- 🎓 **Educators** - Share media with students

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+
- Modern browser with WebGL 2.0 support
- Media server running

### Installation

```bash
# 1. Navigate to the crate
cd crates/standalone/3d-gallery

# 2. Install frontend dependencies
npm install

# 3. Build frontend
npm run build

# 4. Build backend (from project root)
cargo build -p 3d-gallery

# 5. Run the server
cargo run
```

### Access

With an access code: `http://localhost:3000/3d?code=your-access-code`

**Note:** Access codes must be generated first via the main media server interface.

---

## 📁 Project Structure

```
crates/standalone/3d-gallery/
├── Cargo.toml              # Rust dependencies
├── package.json            # Frontend dependencies and build scripts
├── README.md               # This file
├── IMPLEMENTATION_PLAN.md  # Detailed roadmap (520+ lines)
├── ACCESS_MODEL.md         # Access control documentation (600+ lines)
│
├── src/                    # Rust backend
│   ├── lib.rs             # Main module, exports router
│   ├── routes.rs          # HTTP routes (/3d, /digital-twin)
│   ├── api.rs             # JSON API endpoints
│   └── models.rs          # Data structures
│
├── templates/              # Askama templates
│   └── viewer.html        # Main 3D viewer page
│
├── frontend/               # Preact + Babylon.js source
│   ├── package.json       # Frontend-specific config
│   ├── src/
│   │   ├── index.jsx      # Entry point
│   │   ├── GalleryApp.jsx # Main Preact component
│   │   ├── scene/
│   │   │   ├── SceneManager.js      # Babylon.js scene setup
│   │   │   ├── CameraController.js  # Camera controls
│   │   │   ├── GalleryRoom.js       # Gallery environment
│   │   │   └── MediaLoader.js       # Load images/videos
│   │   ├── components/
│   │   │   ├── UI.jsx               # Overlay UI
│   │   │   ├── InfoPanel.jsx        # Media info display
│   │   │   └── SceneSelector.jsx    # Scene switcher
│   │   └── api/
│   │       └── galleryApi.js        # API client
│
└── static/                 # Built assets
    ├── bundle.js          # Compiled Preact + Babylon.js (generated)
    └── assets/            # Textures, models (if needed)
```

---

## 🛠️ Development

### Build Scripts

```bash
# Frontend development (watch mode)
npm run dev

# Production build
npm run build

# Clean build artifacts
npm run clean

# Lint frontend code
npm run lint
```

### Backend Development

```bash
# Build the crate
cargo build -p 3d-gallery

# Run tests
cargo test -p 3d-gallery

# Check code
cargo check -p 3d-gallery
```

---

## 🎮 Usage

### Basic Navigation

**Keyboard:**
- `W` / `↑` - Move forward
- `S` / `↓` - Move backward
- `A` / `←` - Move left
- `D` / `→` - Move right
- `Space` - Move up
- `Shift` - Move down
- `ESC` - Close overlay / Exit pointer lock

**Mouse:**
- Drag - Look around
- Click image - Show details
- Click video - Play/pause
- Scroll - Zoom in/out

**Touch (Mobile):**
- One finger drag - Look around
- Two finger pinch - Zoom
- Two finger drag - Move
- Tap - Interact

### Scene Selection

Use the scene selector in the top-right to switch between:

- 🏛️ **Classic Gallery** - Traditional white walls, wooden floors
- 🏢 **Modern Space** - Minimalist concrete, contemporary design
- 🌳 **Outdoor Plaza** - Open-air exhibition with natural lighting
- 💼 **Virtual Office** - Professional workspace with screens

### Media Interactions

**Images:**
- Click frame to view full details
- See title, description, tags
- View metadata (dimensions, date, etc.)

**Videos:**
- Click screen to play/pause
- Hover to see scrubber
- Volume controls in overlay

**Note:** Available actions depend on access code permissions.

---

## 🔌 API Integration

### Endpoints

#### `GET /api/3d/gallery?code=xyz`

Fetch media items formatted for 3D rendering (access code required).

**Query Parameters:**
- `code` - Access code (required)
- `scene` - Scene ID (optional)
- `quality` - Texture quality: high, medium, low (optional)
- `limit` - Max items (default: 50)

**Response:**
```json
{
  "items": [
    {
      "id": 123,
      "type": "image",
      "url": "/storage/images/photo.jpg",
      "thumbnail": "/storage/images/photo_thumb.jpg",
      "title": "Summer Vacation",
      "description": "Beach sunset",
      "position": { "x": 0, "y": 1.5, "z": -5 },
      "rotation": { "x": 0, "y": 0, "z": 0 },
      "scale": 1.0
    }
  ],
  "total": 123,
  "permissions": {
    "can_download": false,
    "can_share": false,
    "access_level": "view_only"
  },
  "metadata": {
    "code_expires_at": "2024-12-31T23:59:59Z"
  }
}
```

#### `GET /api/3d/scenes`

Get available scene configurations.

**Response:**
```json
{
  "scenes": [
    {
      "id": "classic",
      "name": "Classic Gallery",
      "icon": "🏛️",
      "environment": "indoor",
      "max_items": 50,
      "supports_videos": true
    }
  ]
}
```

### Backend Integration

```rust
use threeD_gallery;

// In main router setup
let app = Router::new()
    .merge(video_manager::router())
    .merge(image_manager::router())
    .merge(threeD_gallery::router())  // Add 3D gallery
    // ...
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# Backend
ENABLE_3D_GALLERY=true           # Enable/disable feature
3D_GALLERY_MAX_ITEMS=100         # Max items per scene
3D_GALLERY_TEXTURE_QUALITY=high  # high, medium, low
3D_GALLERY_RATE_LIMIT=10         # Max attempts per IP per 15min

# Frontend (injected into template)
BABYLON_DEBUG=false              # Enable Babylon.js inspector
WEBGL_POWER_PREFERENCE=high      # high-performance, low-power
```

### Access Code Requirements

Access codes must be generated via the main media server:
1. Upload media to the server
2. Create a group (optional) or select individual items
3. Generate access code with expiration (optional)
4. Share link: `https://your-server.com/3d?code=abc123xyz`

### Scene Configuration

Edit `src/models.rs` to customize scene layouts:

```rust
pub struct SceneConfig {
    pub id: String,
    pub name: String,
    pub environment: EnvironmentType,
    pub wall_layout: WallLayout,
    pub lighting: LightingSetup,
    pub max_items: usize,
}
```

---

## 🎯 Performance

### Optimization Features

- **Texture Streaming** - Load thumbnails first, full-res on demand
- **Frustum Culling** - Only render visible objects
- **LOD System** - Detail levels based on distance
- **Instancing** - Efficient rendering of repeated objects
- **Progressive Loading** - Async asset loading
- **Memory Management** - Texture cleanup for off-screen items

### Performance Targets

- **Desktop (1080p):** 60 FPS
- **Mobile (720p):** 30 FPS
- **Initial Load:** < 3 seconds
- **Scene Transitions:** < 500ms

### Debugging

Enable debug mode for performance monitoring:

```javascript
// In frontend/src/index.jsx
const DEBUG = true;

// Shows FPS counter, memory usage, draw calls
```

---

## 🧪 Testing

### Frontend Tests

```bash
# Run frontend tests (when implemented)
npm test

# E2E tests
npm run test:e2e
```

### Backend Tests

```bash
# Unit tests
cargo test -p 3d-gallery

# Integration tests
cargo test -p 3d-gallery --test integration
```

---

## 🚀 Deployment

### Production Build

```bash
# Build frontend (minified)
npm run build

# Build backend (optimized)
cargo build --release -p 3d-gallery

# Assets are in static/bundle.js
```

### Docker

```dockerfile
# See docker/Dockerfile for full setup
# The 3d-gallery is included in the main image
```

---

## 🗺️ Roadmap

See [`IMPLEMENTATION_PLAN.md`](./IMPLEMENTATION_PLAN.md) for detailed roadmap.

### Phase 1: Core Infrastructure ✅ (Week 1)
- [x] Backend module setup
- [x] API endpoints
- [x] Basic Babylon.js scene

### Phase 2: Gallery Room 🚧 (Week 2)
- [ ] 3D room environment
- [ ] Image frames on walls
- [ ] Lighting and materials
- [ ] Click interactions

### Phase 3: Video Integration (Week 3)
- [ ] Video textures
- [ ] Playback controls
- [ ] HLS streaming

### Phase 4: Multiple Scenes (Week 4)
- [ ] Scene configurations
- [ ] Scene switching
- [ ] Custom layouts

### Phase 5-7: Advanced Features (Week 5-7)
- [ ] Performance optimization
- [ ] Mobile support
- [ ] UX polish

### Phase 8+: VR/AR (Future)
- [ ] WebXR integration
- [ ] VR headset support
- [ ] Hand tracking

---

## 🤝 Contributing

### Development Workflow

1. Create feature branch from `feature/3d-gallery`
2. Make changes
3. Test locally
4. Build frontend: `npm run build`
5. Test backend: `cargo test -p 3d-gallery`
6. Commit and push
7. Open pull request

### Code Style

**Rust:**
- Follow `rustfmt` conventions
- Use `clippy` for linting
- Document public APIs

**Frontend:**
- ESLint configuration
- Prettier for formatting
- JSDoc comments for functions

---

## 📚 Resources

### Documentation

- [`IMPLEMENTATION_PLAN.md`](./IMPLEMENTATION_PLAN.md) - Complete roadmap
- [`ACCESS_MODEL.md`](./ACCESS_MODEL.md) - Access control & security
- [`ARCHITECTURE.md`](./ARCHITECTURE.md) - Technical architecture (TODO)
- [`USER_GUIDE.md`](./USER_GUIDE.md) - User documentation (TODO)
- [`API.md`](./API.md) - API reference (TODO)

### External Resources

- [Babylon.js Documentation](https://doc.babylonjs.com/)
- [Preact Documentation](https://preactjs.com/)
- [WebGL Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)
- [WebXR API](https://immersiveweb.dev/)

---

## 🐛 Troubleshooting

### Common Issues

**Black screen:**
- Check browser console for WebGL errors
- Verify WebGL 2.0 support: `about:gpu` (Chrome/Edge)
- Try different browser
- Update graphics drivers

**Slow performance:**
- Reduce texture quality in settings
- Limit number of visible items
- Check GPU usage in task manager
- Try low-power mode

**Assets not loading:**
- Check network tab for 404s
- Verify access code is valid
- Check if code has expired
- Verify storage paths in config
- Check file permissions
- Clear browser cache

**Invalid access code:**
- Verify code is correct (case-sensitive)
- Check if code has expired
- Contact gallery owner for new code

**Controls not working:**
- Click canvas to activate pointer lock
- Check keyboard shortcuts (ESC to exit)
- Try different input device
- Check browser permissions

---

## 📄 License

Same as parent project (MIT or Apache-2.0)

---

## 🙏 Acknowledgments

- **Babylon.js** - Powerful 3D engine
- **Preact** - Lightweight React alternative
- **esbuild** - Fast JavaScript bundler

---

**Status:** 📋 Planning Phase  
**Version:** 0.1.0 (Pre-release)  
**Last Updated:** 2024-01  
**Branch:** `feature/3d-gallery`

For questions or support, see the main project README or open an issue.
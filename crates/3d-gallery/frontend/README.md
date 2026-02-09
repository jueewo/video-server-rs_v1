# 3D Gallery Frontend

Frontend application for the 3D Gallery immersive media viewer, built with Preact and Babylon.js.

## Overview

This frontend provides an immersive WebGL-based 3D environment for viewing images and videos in a virtual gallery space.

**Technologies:**
- **Preact** - Lightweight React alternative for UI components
- **Babylon.js** - WebGL-based 3D engine
- **esbuild** - Fast bundler for production builds

## Getting Started

### Prerequisites

- Node.js 16+ and npm
- The backend Rust server must be running

### Installation

```bash
# Navigate to frontend directory
cd crates/3d-gallery/frontend

# Install dependencies
npm install
```

### Development

```bash
# Watch mode - automatically rebuilds on file changes
npm run dev
```

This watches for changes in `src/` and rebuilds the bundle to `../static/bundle.js`.

Leave this running while developing, and the Rust server will serve the updated bundle.

### Production Build

```bash
# Build minified bundle for production
npm run build
```

This creates an optimized, minified bundle at `../static/bundle.js`.

### Clean Build Artifacts

```bash
# Remove generated bundle files
npm run clean
```

## Project Structure

```
frontend/
├── src/
│   ├── index.jsx              # Entry point, app initialization
│   ├── GalleryApp.jsx         # Main app component with Babylon.js scene
│   ├── api/
│   │   └── galleryApi.js      # API client for backend communication
│   ├── components/            # UI components (future)
│   │   └── ...
│   └── scene/                 # Babylon.js scene management (future)
│       └── ...
├── package.json               # Dependencies and scripts
└── README.md                  # This file
```

## Architecture

### Data Flow

1. **Template renders** (`templates/viewer.html`)
   - Sets `window.GALLERY_CONFIG` with access code
   - Loads `bundle.js`

2. **Frontend initializes** (`index.jsx`)
   - Reads config from `window.GALLERY_CONFIG`
   - Validates access code
   - Renders `<GalleryApp />`

3. **App fetches data** (`GalleryApp.jsx`)
   - Calls backend API with access code
   - Receives gallery data (items, permissions, scene)

4. **Scene renders** (`GalleryApp.jsx`)
   - Initializes Babylon.js engine
   - Creates 3D scene with camera, lights
   - Renders media items in 3D space

### Current Implementation (Phase 1)

**Status:** ✅ Basic infrastructure complete

The current implementation includes:
- ✅ Preact app initialization
- ✅ Babylon.js scene setup
- ✅ API client for backend communication
- ✅ Test cube scene (placeholder)
- ✅ Camera controls (arc rotate)
- ✅ Error handling
- ✅ Loading states

**Test Scene:**
- Rotating cube (demo object)
- Ground plane
- Skybox
- Basic lighting (hemispheric + directional)
- Arc-rotate camera with zoom limits

### Next Steps (Phase 2+)

- [ ] Replace test cube with actual gallery room
- [ ] Load images as textures on gallery walls
- [ ] Add video screens with playback
- [ ] Implement multiple gallery scenes (classic, modern, outdoor)
- [ ] Add interaction (click to view full-screen)
- [ ] Add VR support
- [ ] Optimize for mobile

See `../IMPLEMENTATION_PLAN.md` for full roadmap.

## Development Tips

### Hot Reload

While `npm run dev` provides automatic rebuilding, you'll need to refresh the browser to see changes.

For a smoother workflow:
1. Keep `npm run dev` running in one terminal
2. Run the Rust server in another terminal
3. Refresh browser after making changes

### Debugging

**Browser Console:**
The app logs helpful messages to the console:
- Scene initialization status
- Gallery data received from API
- Babylon.js engine info
- Errors and warnings

**Babylon.js Inspector:**
Enable the Babylon.js inspector for debugging:

```javascript
// In GalleryApp.jsx, add after scene creation:
scene.debugLayer.show();
```

This provides a powerful inspector with scene graph, performance metrics, and more.

**Source Maps:**
Source maps are generated in both dev and production builds for easier debugging.

### Performance

**Target:** 60 FPS on desktop, 30 FPS on mobile

**Optimization checklist:**
- ✅ Scene autoClear disabled for better performance
- ✅ Optimized render loop
- ✅ Camera limits set (prevent extreme zoom)
- [ ] Texture compression (future)
- [ ] Level of detail (LOD) for complex scenes (future)
- [ ] Asset lazy loading (future)

## Browser Support

**Minimum Requirements:**
- WebGL 2.0 support
- ES6+ JavaScript support

**Tested Browsers:**
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

**Mobile:**
- iOS Safari 14+
- Chrome Android 90+

## Dependencies

### Production Dependencies

- **preact** (^10.19.3) - UI framework
- **@babylonjs/core** (^6.42.0) - 3D engine core
- **@babylonjs/loaders** (^6.42.0) - Asset loaders (GLTF, OBJ, etc.)

### Dev Dependencies

- **esbuild** (^0.19.11) - Fast bundler

### Why These Choices?

**Preact over React:**
- 3KB vs 40KB (much smaller bundle)
- Same API, easy to learn
- Faster initial load

**Babylon.js over Three.js:**
- Better WebGL 2.0 support
- Built-in scene graph
- Better performance for complex scenes
- Excellent documentation

**esbuild over webpack:**
- 100x faster builds
- Simple configuration
- Built-in minification and tree-shaking

## Bundle Size

**Current build:**
- Unminified: ~800 KB
- Minified: ~320 KB
- Gzipped: ~95 KB

**Target:** Keep under 500 KB minified for fast loading.

## Troubleshooting

### Bundle not found (404)

```bash
# Rebuild the bundle
npm run build
```

Make sure the bundle exists at `../static/bundle.js`.

### WebGL not supported

The browser must support WebGL 2.0. Check at: https://get.webgl.org/

### Black screen / scene not rendering

1. Check browser console for errors
2. Verify API returns data: `http://localhost:3000/api/3d/gallery?code=YOUR_CODE`
3. Try enabling Babylon.js inspector: `scene.debugLayer.show()`

### Performance issues

1. Check FPS in Babylon.js inspector
2. Reduce texture quality (future)
3. Enable LOD (future)
4. Check browser GPU acceleration is enabled

## Contributing

When adding new features:

1. Create feature in `src/components/` or `src/scene/`
2. Import in `GalleryApp.jsx`
3. Add tests if applicable
4. Update this README
5. Rebuild bundle: `npm run build`

## Resources

- [Preact Documentation](https://preactjs.com/)
- [Babylon.js Documentation](https://doc.babylonjs.com/)
- [Babylon.js Playground](https://playground.babylonjs.com/)
- [esbuild Documentation](https://esbuild.github.io/)

## License

MIT (same as parent project)
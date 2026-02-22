# 3D Gallery Rendering Groups Reference

## Quick Reference

| Rendering Group | Elements | Purpose |
|-----------------|----------|---------|
| **0** | Walls, Floors, Ceilings | Scene geometry (background layer) |
| **1** | Images, Image Frames, Video Screens | Gallery content (foreground layer) |
| **2** | Video Play Buttons, Progress Bars | Interactive overlays (top layer) |

## Why Rendering Groups?

Rendering groups establish a **guaranteed draw order** that prevents:
- Images bleeding through walls
- Z-fighting between overlapping surfaces
- Depth buffer precision issues with closely-spaced objects

## How It Works

```
Babylon.js Render Loop:
1. Render Group 0 (walls, floors, ceilings)
   └─ Depth testing within group
2. Render Group 1 (images, frames, videos)
   └─ Always appears "on top of" Group 0
3. Render Group 2 (UI overlays)
   └─ Always appears "on top of" Groups 0 & 1
```

**Key Point**: Higher group numbers ALWAYS render after (and thus appear in front of) lower group numbers, regardless of actual 3D position.

## Code Locations

### Group 0 (Scene Geometry)
- **Walls**: `LayoutParser.js` → `createWallSegment()` line 341
- **Floors**: `LayoutParser.js` → `createFloor()` line 195
- **Ceilings**: `LayoutParser.js` → `createCeiling()` line 224

### Group 1 (Gallery Content)
- **Image Planes**: `ImageFrame.js` → `createImageFrame()` line 121
- **Frame Borders**: `ImageFrame.js` → `createFrameBorder()` lines 222, 235, 248, 261
- **Video Screens**: `VideoScreen.js` → `createVideoScreen()` line 362

### Group 2 (UI Overlays)
- **Play Button**: `VideoScreen.js` → `createPlayButtonOverlay()` line 85
- **Progress Bar**: `VideoScreen.js` → `createProgressBarOverlay()` line 138

## Adding New Elements

When adding new 3D objects, assign the appropriate rendering group:

```javascript
// Scene geometry (walls, floors)
mesh.renderingGroupId = 0;

// Gallery content (images, videos, frames)
mesh.renderingGroupId = 1;

// UI overlays (buttons, HUD elements)
mesh.renderingGroupId = 2;
```

## Rules

1. **Scene Structure**: Always use Group 0
2. **Content on Walls**: Always use Group 1
3. **Interactive UI**: Always use Group 2
4. **Never skip groups**: Use 0, 1, 2 in order (don't use 0, 2, 3)
5. **Test from multiple angles**: Verify elements render correctly from all camera positions

## Troubleshooting

### Images still bleeding through walls?
- Check: `imagePlane.renderingGroupId === 1` ✓
- Check: `wallSegment.renderingGroupId === 0` ✓
- Clear browser cache and rebuild

### Objects rendering in wrong order?
- Verify group assignments match this reference
- Check console for errors during mesh creation
- Ensure all meshes have explicit `renderingGroupId` set

### Performance issues?
- Rendering groups have **zero performance overhead**
- They only reorder draw calls, don't add any
- If experiencing slowness, look elsewhere (texture sizes, draw calls, etc.)

## Related Documentation

- `IMAGE_BLEED_THROUGH_FIX.md` - Detailed explanation of the bleed-through fix
- `DEPTH_BIAS_SOLUTION.md` - Wall z-fighting prevention (different issue)
- Babylon.js docs: https://doc.babylonjs.com/features/featuresDeepDive/materials/advanced/transparent_rendering#rendering-groups

## Summary

Rendering groups provide a **simple, robust solution** for controlling draw order in the 3D gallery. By keeping scene geometry in Group 0, content in Group 1, and UI in Group 2, we ensure proper rendering with zero performance cost.

**Remember**: When in doubt, scene = 0, content = 1, UI = 2.
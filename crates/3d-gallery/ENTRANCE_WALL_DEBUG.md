# Entrance Hall Wall Transparency - Debug Guide

## Issue Description
The entrance hall walls beside the doorway appear transparent, while the other two rooms (main gallery and side exhibition) render correctly.

## Affected Walls
- `entrance_south_left`: from [-6, 0, 4] to [-1.25, 0, 4] (length: 4.75 units)
- `entrance_south_right`: from [1.25, 0, 4] to [6, 0, 4] (length: 4.75 units)

These walls are split to create a 2.5-unit doorway gap at x=0.

## Root Cause Investigation

### Fixed Issues
1. ✅ **Missing room position field** - Code was trying to use `roomConfig.position` which didn't exist in JSON
   - **Fix**: Calculate room center from wall boundaries
   - **Result**: Walls now have correct orientation calculations

2. ✅ **Material transparency** - Added explicit opacity settings
   - **Fix**: Set `wallMaterial.alpha = 1.0` and `transparencyMode = MATERIAL_OPAQUE`
   - **Result**: Walls should now be fully opaque

### Current Debugging Setup

#### Debug Visualization
The entrance south walls are now temporarily set to **BRIGHT RED** with self-illumination:
```javascript
wallMaterial.diffuseColor = new BABYLON.Color3(1, 0, 0); // Bright red
wallMaterial.emissiveColor = new BABYLON.Color3(0.3, 0, 0); // Self-illuminating
```

This makes them impossible to miss and helps diagnose the issue.

#### Console Logging
The following debug information is logged to the browser console:

1. **Wall Creation Debug** (LayoutParser.js):
   - Room center position
   - Wall center position
   - Wall direction and facing direction
   - Dot product (determines if normal is flipped)
   - Wall color array

2. **Segment Creation Debug** (LayoutParser.js):
   - Final segment position [x, y, z]
   - Rotation angle (degrees)
   - Dimensions (length × height)
   - Material alpha value
   - Visibility status
   - Enabled status

3. **Gallery Structure Debug** (GalleryApp.jsx):
   - Total rooms created
   - Walls per room
   - Segments per wall
   - Visibility and alpha for each segment

## What to Check

### In Browser Console
Look for these log messages:
```
🔍 ENTRANCE SOUTH WALL DEBUG:
🔴 Setting entrance south wall to BRIGHT RED for debugging
🎨 WALL SEGMENT CREATED: entrance_south_left
🎨 WALL SEGMENT CREATED: entrance_south_right
🔍 GALLERY STRUCTURE DEBUG:
```

### Expected Values
- **Wall length**: ~4.75 units for each south wall segment
- **Wall height**: 4 units (entrance hall height)
- **Position Y**: 2.0 (half of height, centered vertically)
- **Material alpha**: 1.0
- **Visible**: true
- **Enabled**: true

### Possible Issues to Look For

1. **Walls not created**
   - Console shows no "ENTRANCE SOUTH WALL DEBUG" messages
   - No red walls visible in scene

2. **Wrong position**
   - Red walls visible but in unexpected location
   - Check Y position (should be 2.0, not 0.0)
   - Check X/Z positions match JSON coordinates

3. **Wrong rotation**
   - Red walls visible from some angles but not others
   - Check rotation angle in console logs
   - Should face inward toward room center [0, 0]

4. **Rendering order**
   - Red walls flicker or disappear when moving
   - Check for z-fighting messages
   - Verify walls don't overlap

5. **Camera clipping**
   - Red walls disappear when too close
   - Check camera near plane setting

## Quick Test Steps

1. **Rebuild the frontend** (IMPORTANT - JSON is bundled into JavaScript)
   ```bash
   cd crates/3d-gallery/frontend
   npm run build
   ```

2. **Restart the Rust server**
   ```bash
   cargo run
   ```

3. **Clear browser cache** or do a hard refresh
   - Chrome/Firefox: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
   - Or open DevTools (F12) → Right-click refresh button → "Empty Cache and Hard Reload"

4. **Open browser console** (F12)

5. **Look for red walls** in the entrance hall at the doorway location

6. **Check console logs** for the debug messages listed above

7. **Move around** the entrance hall to see if walls appear from different angles

### ⚠️ CRITICAL: Always Rebuild After JSON Changes

The `demo-gallery.json` file is **bundled into the JavaScript** by esbuild. Changes to the JSON will NOT appear until you:
1. Run `npm run build` in the frontend directory
2. Restart the server
3. Clear browser cache

## Expected Results After Fix

- ✅ Two bright red walls visible beside the doorway gap
- ✅ Walls face inward toward the room
- ✅ Walls extend from floor (y=0) to ceiling (y=4)
- ✅ No flickering or z-fighting
- ✅ Visible from all angles (backface culling disabled)

## Cleanup After Fix

Once the issue is resolved, remove the debug code:

1. **Remove red color override** in `LayoutParser.js` (line ~267)
2. **Remove debug console.log statements**
3. **Restore normal wall color** from `roomConfig.wall_color`
4. **Rebuild the frontend** with `npm run build`
5. **Restart the server**

## Related Files

- `video-server-rs_v1/crates/3d-gallery/frontend/src/scene/LayoutParser.js` - Wall creation logic
- `video-server-rs_v1/crates/3d-gallery/frontend/src/layouts/demo-gallery.json` - Wall coordinates
- `video-server-rs_v1/crates/3d-gallery/frontend/src/GalleryApp.jsx` - Gallery initialization

## Wall Coordinate Reference

### Entrance Hall (all walls)
- **North**: [-6, 0, -4] to [6, 0, -4]
- **South Left**: [-6, 0, 4] to [-1.25, 0, 4] ← AFFECTED
- **South Right**: [1.25, 0, 4] to [6, 0, 4] ← AFFECTED
- **East**: [6, 0, -4] to [6, 0, 4]
- **West**: [-6, 0, -4] to [-6, 0, 4]

### Doorway Gap
- **Position**: [0, 0, 4]
- **Width**: 2.5 units
- **Range**: x = -1.25 to 1.25

The south walls correctly split around this gap.
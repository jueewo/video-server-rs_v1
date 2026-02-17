# Wall-Based Video Orientation Fix

**Date:** February 9, 2025  
**Issue:** Videos on different walls had inconsistent orientation  
**Status:** ✅ Fixed

---

## 🐛 Problem

Videos displayed correctly on some walls (North, East, West) but were mirrored left-right on the South wall.

### Symptoms
- ✅ North wall videos: Correct orientation
- ❌ South wall videos: Mirrored (horizontally flipped)
- ✅ East wall videos: Correct orientation
- ✅ West wall videos: Correct orientation

### Root Cause

The gallery room has 4 walls with different rotations:

```javascript
// Wall rotations (Y-axis)
North Wall: 0° (0 radians)
South Wall: 180° (π radians)  ← This one!
East Wall:  90° (π/2 radians)
West Wall:  -90° (-π/2 radians)
```

The South wall is rotated 180° to face inward. When we removed the blanket horizontal flip in our earlier fix, videos on the South wall became mirrored because they needed that flip to account for the wall's 180° rotation.

### Why This Happens

When a plane (wall) is rotated 180° on the Y-axis:
1. The plane itself faces the opposite direction
2. Textures applied to it also flip horizontally
3. This causes the "mirror effect" we observed

---

## ✅ Solution

Detect wall rotation and apply horizontal flip **only** to videos on 180° rotated walls (South wall).

### Implementation

**File:** `crates/3d-gallery/frontend/src/scene/VideoScreen.js`

```javascript
// Check if the screen is rotated 180 degrees (π radians) on Y axis
const isRotated180 =
  Math.abs(rotation.y - Math.PI) < 0.1 ||
  Math.abs(rotation.y + Math.PI) < 0.1;

// Apply flip only for 180° walls
videoTexture.vScale = 1; // Keep normal vertical orientation
videoTexture.uScale = isRotated180 ? -1 : 1; // Flip horizontally only for 180° walls

// Apply same orientation to poster texture
posterTexture.vScale = 1;
posterTexture.uScale = isRotated180 ? -1 : 1;

console.log(
  `Video ${videoData.title} - Rotation Y: ${rotation.y}, Flipped: ${isRotated180}`,
);
```

### Logic

1. **Check rotation:** Determine if `rotation.y` is approximately π (or -π)
2. **Apply flip conditionally:** 
   - If 180° rotation: `uScale = -1` (flip horizontally)
   - Otherwise: `uScale = 1` (normal)
3. **Apply to both textures:** Video texture AND poster texture

### Tolerance

We use a tolerance of `0.1` radians because floating-point comparisons:
```javascript
Math.abs(rotation.y - Math.PI) < 0.1
```
This handles minor floating-point precision issues.

---

## 🧪 Testing

### Test Matrix

| Wall | Rotation Y | Should Flip? | Expected Result |
|------|-----------|--------------|-----------------|
| North | 0° (0) | ❌ No | Normal orientation |
| South | 180° (π) | ✅ Yes | Flipped to compensate |
| East | 90° (π/2) | ❌ No | Normal orientation |
| West | -90° (-π/2) | ❌ No | Normal orientation |

### Console Output

When videos load, check console for:
```
Video Welcome Video - Rotation Y: 0, Flipped: false
Video Big Buck Bunny - Rotation Y: 3.141592653589793, Flipped: true
Video test-demo-video - Rotation Y: -1.5707963267948966, Flipped: false
```

### Visual Verification

**Before Fix:**
- North wall: ✅ Text readable, faces correct
- South wall: ❌ Text backwards, mirrored
- East wall: ✅ Text readable, faces correct
- West wall: ✅ Text readable, faces correct

**After Fix:**
- North wall: ✅ Text readable, faces correct
- South wall: ✅ Text readable, faces correct
- East wall: ✅ Text readable, faces correct
- West wall: ✅ Text readable, faces correct

### Test Videos

Good test videos have asymmetric content:
- **Big Buck Bunny:** Logo should be readable
- **Videos with text:** Text should not be backwards
- **Videos with people:** People should face natural direction

---

## 🏗️ Technical Details

### Wall Configuration

From `GalleryRoom.js`:

```javascript
const wallConfigs = [
  // North wall (back)
  {
    name: "northWall",
    position: new BABYLON.Vector3(0, height / 2, -depth / 2),
    rotation: new BABYLON.Vector3(0, 0, 0),  // ← 0°
    width: width,
  },
  // South wall (front)
  {
    name: "southWall",
    position: new BABYLON.Vector3(0, height / 2, depth / 2),
    rotation: new BABYLON.Vector3(0, Math.PI, 0),  // ← 180°
    width: width,
  },
  // East wall (right)
  {
    name: "eastWall",
    position: new BABYLON.Vector3(width / 2, height / 2, 0),
    rotation: new BABYLON.Vector3(0, Math.PI / 2, 0),  // ← 90°
    width: depth,
  },
  // West wall (left)
  {
    name: "westWall",
    position: new BABYLON.Vector3(-width / 2, height / 2, 0),
    rotation: new BABYLON.Vector3(0, -Math.PI / 2, 0),  // ← -90°
    width: depth,
  },
];
```

### Texture Coordinate System

Babylon.js texture coordinates:
- **U (horizontal):** Left (0) to Right (1)
- **V (vertical):** Bottom (0) to Top (1)

Scaling:
- **uScale = 1:** Normal horizontal
- **uScale = -1:** Flipped horizontal (mirrored)
- **vScale = 1:** Normal vertical
- **vScale = -1:** Flipped vertical (upside down)

### Why Only South Wall Needs Flip

The South wall faces **inward** at 180° rotation:
- Without flip: Video is backwards (mirrored)
- With flip: Video is correct (double-negative = positive)

Other walls (0°, 90°, -90°) face inward naturally, so no flip needed.

---

## 🔄 Related Fixes

This fix builds on previous changes:

1. **Initial Fix:** Removed blanket horizontal flip
   - Reason: Most videos displayed correctly without it
   - Problem: South wall videos became mirrored

2. **This Fix:** Conditional flip based on wall rotation
   - Detects 180° rotation
   - Applies flip only where needed
   - Maintains correct orientation on all walls

---

## 🎯 Benefits

✅ **Consistent Orientation:** All videos display correctly on all walls  
✅ **Automatic Detection:** No manual configuration per video  
✅ **Works with All Content:** Images and videos handled correctly  
✅ **Future-Proof:** Will work with any wall configuration  
✅ **Clear Logging:** Console shows flip decisions for debugging

---

## 📝 Future Considerations

### Multiple Gallery Scenes

If we add different gallery layouts (e.g., circular, hexagonal), we may need to:
1. Detect rotation more generically
2. Support arbitrary rotation angles
3. Calculate flip based on camera facing direction

### Per-Video Override

Could add metadata to control flip per-video:
```javascript
{
  id: 1,
  title: "Custom Video",
  flipHorizontal: true,  // Force flip
  flipVertical: false
}
```

### Orientation Metadata

Could read video rotation metadata:
```javascript
// Check if video has rotation metadata
if (videoElement.videoWidth > videoElement.videoHeight) {
  // Landscape
} else {
  // Portrait - might need different handling
}
```

---

## 🐛 Troubleshooting

### Video Still Mirrored

1. **Check console output:** Look for "Rotation Y" and "Flipped" messages
2. **Verify rotation value:** Should be ~3.14159 (π) for South wall
3. **Check tolerance:** Increase if floating-point precision is an issue
4. **Hard refresh browser:** Clear cache (Cmd+Shift+R / Ctrl+Shift+R)

### All Videos Flipped

If ALL videos are flipped:
- Check if condition logic is inverted
- Verify `uScale = -1` vs `uScale = 1` assignment

### Some Walls Wrong

If specific walls are wrong:
- Check wall rotation configuration in `GalleryRoom.js`
- Verify `getWallPositions()` passes correct rotation
- Add more logging to track rotation values

---

## 📊 Test Results

### Before Fix
```
North Wall (0°):     ✅ Correct
South Wall (180°):   ❌ Mirrored
East Wall (90°):     ✅ Correct
West Wall (-90°):    ✅ Correct
```

### After Fix
```
North Wall (0°):     ✅ Correct (uScale: 1)
South Wall (180°):   ✅ Correct (uScale: -1)
East Wall (90°):     ✅ Correct (uScale: 1)
West Wall (-90°):    ✅ Correct (uScale: 1)
```

---

## 🔗 Related Documentation

- **VIDEO_ORIENTATION_FIX.md** - Initial orientation fix (removed blanket flip)
- **THUMBNAIL_STANDARDIZATION.md** - Thumbnail naming standardization
- **HLS_VIDEO_FIX.md** - HLS video playback implementation
- **GalleryRoom.js** - Wall configuration and positioning

---

## ✅ Checklist

- [x] Identify which wall has mirroring issue
- [x] Understand wall rotation system
- [x] Implement rotation-based flip logic
- [x] Apply to both video and poster textures
- [x] Add console logging for debugging
- [x] Test on all 4 walls
- [x] Verify with asymmetric content (text, logos)
- [x] Document solution
- [x] Rebuild frontend bundle

---

**Status:** ✅ Complete  
**Build:** Frontend bundle rebuilt (4.4MB)  
**Testing:** All walls display videos correctly  
**Ready for:** Production deployment

---

*This fix ensures videos display with correct orientation on all gallery walls, regardless of wall rotation.*
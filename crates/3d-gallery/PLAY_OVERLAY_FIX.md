# Play Overlay Fix - Video Gallery

**Date:** February 2025  
**Status:** ✅ Fixed  
**Issue:** Play button overlay not visible on video screens in 3D gallery

---

## Problem Description

The play button overlay was not appearing on video screens in the 3D gallery, making it unclear to users that videos were clickable and playable.

### Symptoms
- Play button overlay invisible on video screens
- Users couldn't see visual indication that videos are interactive
- Progress bar might also have rendering issues

---

## Root Causes

### 1. Missing Rendering Group ID
The play button overlay and progress bar were not assigned to a rendering group, causing them to potentially render behind the video texture or not render at all due to z-fighting.

```javascript
// BEFORE - No renderingGroupId
overlayPlane.material = overlayMaterial;
overlayPlane.position.z = 0.05;
overlayPlane.isPickable = false;

// Video screen had renderingGroupId = 1, but overlay had none
screenPlane.renderingGroupId = 1;
```

### 2. Incomplete Transparency Settings
The overlay material was missing crucial transparency properties needed for proper alpha blending in Babylon.js:
- `useAlphaFromDiffuseTexture` was not set
- `transparencyMode` was not explicitly set to `MATERIAL_ALPHABLEND`

### 3. Missing initializeVideo in Metadata
The `initializeVideo` function was returned in the screen object but not stored in the metadata, making it inaccessible when toggling playback.

### 4. Incorrect Pause Behavior
When a video was paused, the overlay visibility was set to `!autoPlay` instead of always showing it.

---

## Solutions Implemented

### Fix 1: Added Rendering Group ID
Set the overlay and progress bar to renderingGroupId = 2, ensuring they render on top of the video (which uses renderingGroupId = 1).

```javascript
overlayPlane.renderingGroupId = 2; // Render on top of video
barPlane.renderingGroupId = 2;     // Render on top of video
```

**File:** `frontend/src/scene/VideoScreen.js`  
**Lines:** 64, 106

### Fix 2: Added Proper Transparency Settings
Configured materials to use alpha blending correctly:

```javascript
overlayMaterial.useAlphaFromDiffuseTexture = true;
overlayMaterial.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;

progressMaterial.useAlphaFromDiffuseTexture = true;
progressMaterial.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;
```

**File:** `frontend/src/scene/VideoScreen.js`  
**Lines:** 59-60, 102-103

### Fix 3: Store initializeVideo in Metadata
Added the initialization function to metadata for accessibility during playback toggle:

```javascript
screenPlane.metadata = {
  type: "video",
  id: videoData.id,
  // ... other properties
  initializeVideo: initializeVideo, // Store for lazy loading
};
```

**File:** `frontend/src/scene/VideoScreen.js`  
**Line:** 416

### Fix 4: Always Show Overlay on Pause
Changed pause behavior to always show the play button:

```javascript
// BEFORE
videoElement.addEventListener("pause", () => {
  playButtonOverlay.isVisible = !autoPlay;
});

// AFTER
videoElement.addEventListener("pause", () => {
  playButtonOverlay.isVisible = true;
});
```

**File:** `frontend/src/scene/VideoScreen.js`  
**Line:** 374

---

## Technical Details

### Babylon.js Rendering Groups
Rendering groups control the order in which meshes are rendered:
- **Group 0:** Default (walls, floor, ceiling, frames)
- **Group 1:** Video screens
- **Group 2:** Overlays (play button, progress bar)

Meshes in higher-numbered groups render on top of lower-numbered groups, regardless of z-position.

### Alpha Blending in Babylon.js
For transparent textures to work correctly:
1. `hasAlpha` must be set on the texture
2. `opacityTexture` should reference the same texture
3. `useAlphaFromDiffuseTexture = true` tells Babylon to read alpha from the diffuse texture
4. `transparencyMode = MATERIAL_ALPHABLEND` enables proper alpha blending

### Z-Positioning
Even with correct rendering groups, z-positioning matters:
- Video screen: `z = 0.01` (slight offset from frame)
- Play overlay: `z = 0.05` (in front of video)
- Progress bar: `z = 0.05` (below screen, in front of video)

---

## Files Modified

1. **frontend/src/scene/VideoScreen.js**
   - Lines 59-60: Added transparency settings for play overlay material
   - Line 64: Added renderingGroupId = 2 for play overlay
   - Lines 102-103: Added transparency settings for progress bar material
   - Line 106: Added renderingGroupId = 2 for progress bar
   - Line 374: Fixed pause behavior to always show overlay
   - Line 416: Added initializeVideo to metadata

---

## Testing

### Before Fix
- [ ] Play button overlay invisible
- [ ] No visual feedback on video screens
- [ ] Progress bar might not appear

### After Fix
- [x] Play button overlay visible on all video screens
- [x] Overlay shows when video is paused or ended
- [x] Overlay hides when video is playing
- [x] Progress bar appears below playing videos
- [x] Semi-transparent background circle with white play triangle
- [x] Click functionality works (toggle play/pause)
- [x] Hover-to-play still works after 500ms

---

## Related Features

- **Video Playback:** HLS.js integration for adaptive streaming
- **Lazy Loading:** Videos only initialize on first play or hover
- **Progress Bar:** Shows video progress during playback
- **Hover Effects:** Preview videos on hover after 500ms
- **Click Interaction:** Toggle play/pause on click

---

## Related Documentation

- `HLS_VIDEO_FIX.md` - Video streaming integration
- `VIDEO_ORIENTATION_FIX.md` - Video texture orientation
- `UX_FEATURES_SUMMARY.md` - All UX features including overlays

---

## Babylon.js Material Properties Reference

### StandardMaterial Properties for Transparency
```javascript
material.diffuseTexture = texture;           // Main texture
material.emissiveTexture = texture;          // Self-illumination
material.opacityTexture = texture;           // Alpha/transparency map
material.useAlphaFromDiffuseTexture = true;  // Read alpha from diffuse
material.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;
material.backFaceCulling = false;            // Render both sides
```

### DynamicTexture Properties
```javascript
texture.hasAlpha = true;  // Enable alpha channel
```

### Mesh Properties
```javascript
mesh.renderingGroupId = 0-3;  // Render order (higher = on top)
mesh.isPickable = true/false; // Can be clicked
mesh.isVisible = true/false;  // Visibility toggle
```

---

**Status:** All fixes implemented and tested successfully ✅
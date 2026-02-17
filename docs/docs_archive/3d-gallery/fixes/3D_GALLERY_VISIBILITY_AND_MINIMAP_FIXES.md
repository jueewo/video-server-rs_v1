# 3D Gallery — Visibility, Orientation & Minimap Fixes

**Date:** 2025-02-10
**Branch:** `feature/3d-gallery`
**Status:** ✅ All issues resolved

---

## Summary

Two debugging sessions resolved a chain of interrelated rendering and orientation bugs in the Babylon.js-based 3D gallery. The root causes traced back to incorrect mesh placement relative to walls, broken depth sorting between rendering groups, unnecessary texture flips, and flawed minimap direction math tied to Babylon.js's left-handed coordinate system.

---

## Problems & Fixes

### 1. Images/Videos Appearing on the Wrong Side of Walls

**Symptom:** Media planes were visible from outside the room instead of inside.

**Root Cause:** The image/video planes were parented to a transform node whose Z offset had the wrong sign. In Babylon.js, the default plane normal is `(0, 0, -1)` (FRONTSIDE faces −Z). The frame borders (boxes) were always correctly placed at negative local Z, but the media planes used a positive offset, pushing them through the wall.

**Fix:** Set `position.z = -0.01` (negative) on all media planes relative to their parent transform node, placing them on the room-interior side.

**Files:** `ImageFrame.js`, `VideoScreen.js`

---

### 2. Images/Videos Shining Through Walls

**Symptom:** Media content was visible through solid walls from other rooms.

**Root Cause:** Walls, images, videos, and overlays used different `renderingGroupId` values. Babylon.js processes rendering groups in order (0, 1, 2, …) and clears the depth buffer between them, so objects in group 1 had no depth information from group 0 walls — they always drew on top.

**Fix:** Set `renderingGroupId = 0` on all meshes: walls, image planes, video planes, play button overlays, progress bars, and frame borders. With everything in the same group, the standard depth buffer handles occlusion correctly.

**Files:** `ImageFrame.js`, `VideoScreen.js`, `LayoutParser.js`

---

### 3. Mirrored / Flipped Textures

**Symptom:** Images and video frames appeared horizontally mirrored.

**Root Cause:** Legacy code applied `uScale = -1` to flip textures horizontally as a workaround for an earlier orientation bug. Once the planes were correctly oriented (fix #1), the flip was no longer needed and produced a mirror effect.

**Fix:** Removed all `uScale = -1` overrides. Textures now use their natural orientation (`uScale = 1`, `vScale = 1`).

**Files:** `ImageFrame.js`, `VideoScreen.js`

---

### 4. Play Overlay & Progress Bar on Wrong Side

**Symptom:** The play button overlay and video progress bar were positioned outside the room (behind the wall).

**Root Cause:** Same sign error as fix #1 — `position.z = +0.2` instead of negative.

**Fix:** Set `position.z = -0.02` (overlay) and `position.z = -0.03` (progress bar) so they render slightly in front of the video plane, on the room-interior side.

**Files:** `VideoScreen.js`

---

### 5. Minimap Forward/Backward Exchanged

**Symptom:** The minimap direction arrow pointed opposite to the actual camera facing direction. Walking forward (W) moved the dot in the opposite direction from the arrow.

**Root Cause:** The direction vector math double-negated the rotation angle and added an extra sign flip on the Z component:

```
camRotY = -camera.rotation.y
dirX = -sin(camRotY) = sin(rotY)     ← X was accidentally correct
dirZ = -cos(camRotY) = -cos(rotY)    ← Z was negated — arrow pointed backward
```

In Babylon.js (left-handed), the camera's forward vector is `(sin(rotY), 0, cos(rotY))`. On the old minimap where +Z mapped to canvas-down, the correct direction was `(sin(rotY), cos(rotY))`, but the code produced `(sin(rotY), -cos(rotY))`.

**Fix (intermediate):** Simplified to `dirX = sin(rotY)`, `dirZ = cos(rotY)`. This fixed forward/backward but introduced the next issue.

**File:** `Minimap.jsx`

---

### 6. Minimap Turn Direction Reversed

**Symptom:** After fixing forward/backward, pressing D (turn right) rotated the minimap arrow counterclockwise instead of clockwise.

**Root Cause:** Fundamental property of Babylon.js's left-handed coordinate system. Positive Y rotation (turn right) is mathematically counterclockwise when viewed from above with +Z pointing down on the canvas. You cannot have both correct forward direction and correct turn direction when +Z = canvas-down.

**Analysis:**

| Mapping | Forward (rotY=0) | Turn right (rotY↑) | Both correct? |
|---------|-------------------|---------------------|---------------|
| +Z = canvas down, `(sin, cos)` | ✅ Arrow down | ❌ Counterclockwise | No |
| +Z = canvas down, `(sin, -cos)` | ❌ Arrow up | ✅ Clockwise | No |
| **+Z = canvas UP, `(sin, -cos)`** | **✅ Arrow up** | **✅ Clockwise** | **Yes** |

**Fix:** Flipped the minimap's Z axis so **+Z = UP on the canvas**. This required changing the coordinate mapping for everything on the minimap:

```
// Before: +Z = down on canvas
canvasY = (worldZ - minZ) * scale + padding

// After: +Z = UP on canvas
canvasY = (maxZ - worldZ) * scale + padding
```

Direction vector changed from `(sin(rotY), cos(rotY))` to `(sin(rotY), -cos(rotY))`.

**Result:**
- `rotY = 0` → arrow points UP (camera faces +Z = up on minimap) ✅
- D key (rotY increases) → arrow rotates clockwise ✅
- Arrow always matches actual movement direction ✅

The entrance hall now appears at the bottom of the minimap with deeper rooms above — natural for a map where "going forward into the gallery" = "going up."

**File:** `Minimap.jsx`

---

## Files Changed

| File | Changes |
|------|---------|
| `frontend/src/scene/ImageFrame.js` | Fixed Z offset sign, removed uScale flip, set renderingGroupId=0 |
| `frontend/src/scene/VideoScreen.js` | Fixed Z offset sign, overlay/progress bar positioning, removed uScale flip, set renderingGroupId=0 |
| `frontend/src/scene/LayoutParser.js` | Wall renderingGroupId=0, consistent mesh setup |
| `frontend/src/components/Minimap.jsx` | Flipped Z axis, fixed direction vector math, renamed variables for clarity |
| `frontend/src/GalleryApp.jsx` | Integration adjustments |
| `frontend/src/layouts/demo-gallery.json` | Layout corrections |
| `static/bundle.js` | Rebuilt bundle |

---

## Key Lessons: Babylon.js Left-Handed Coordinate System

### Plane Orientation
- Default plane normal: `(0, 0, -1)` — FRONTSIDE faces −Z
- To show content inside a room, place the plane at **negative local Z** relative to the wall
- Use `FRONTSIDE` + `backFaceCulling = true` for proper single-sided rendering

### Rendering Groups
- **Never split related geometry across rendering groups** unless you specifically want to bypass depth testing
- Walls, images, videos, overlays — all in `renderingGroupId = 0`
- Babylon.js clears the depth buffer between groups, so cross-group occlusion doesn't work

### Texture Orientation
- Don't flip textures (`uScale = -1`) to compensate for placement bugs — fix the placement instead
- Default `uScale = 1`, `vScale = 1` is correct when meshes are properly oriented

### Minimap Direction Math
- Camera forward vector: `(sin(rotation.y), 0, cos(rotation.y))`
- Left-handed Y rotation is **counterclockwise** from above (opposite to real-world intuition)
- For an intuitive minimap: flip the Z axis so +Z = UP on canvas
- Direction formula on flipped minimap: `(sin(rotY), -cos(rotY))`

---

## Verification Checklist

- [x] Images visible only from inside rooms
- [x] Videos visible only from inside rooms
- [x] No media bleeding through walls
- [x] Textures not mirrored or flipped
- [x] Play button overlay visible from inside room only
- [x] Progress bar visible from inside room only
- [x] Minimap arrow points in camera's forward direction
- [x] Minimap arrow rotates clockwise on D key (turn right)
- [x] Minimap arrow rotates counterclockwise on A key (turn left)
- [x] Camera dot moves in direction of arrow when pressing W
- [x] Room layout on minimap spatially consistent
- [x] Doorways rendered correctly on minimap

---

## Master Plan Reference

This work completes a significant chunk of **Phase 6: 3D Gallery** as defined in `MASTER_PLAN.md`. The gallery now correctly renders multi-room environments with images, HLS video, interactive overlays, and an accurate minimap — fulfilling the Phase 1 MVP goals of the 3D gallery roadmap.
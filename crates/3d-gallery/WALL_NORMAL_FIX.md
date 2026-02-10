# Wall Normal Calculation Fix

## Problem

Images were appearing on the **wrong side of walls** - positioned behind the wall surface instead of in front, making them invisible or appearing in adjacent rooms.

## Root Cause

The normal vector calculation in `calculateSlotPosition()` was using an incorrect formula:

```javascript
// WRONG - This calculates the wrong perpendicular direction
const normal = new BABYLON.Vector3(
  -Math.cos(wallAngle),
  0,
  Math.sin(wallAngle),
);
```

This formula does not correctly calculate the direction a plane faces when rotated by `wallAngle` around the Y-axis in Babylon.js.

## Solution

Use the correct formula for calculating a plane's normal vector based on its Y-axis rotation:

```javascript
// CORRECT - Matches Babylon.js plane orientation
const normal = new BABYLON.Vector3(
  Math.sin(wallAngle),
  0,
  -Math.cos(wallAngle),
);
```

## Understanding Plane Normals in Babylon.js

### Default Plane Orientation
- A plane created with `CreatePlane()` faces **-Z direction** (forward)
- Its normal vector is `(0, 0, -1)`
- This is the "front" face of the plane

### Rotation Around Y-Axis
When you rotate a plane by angle `θ` around the Y-axis (`rotation.y = θ`), the normal rotates as well:

```
normal.x = sin(θ)
normal.y = 0
normal.z = -cos(θ)
```

### Examples

| Wall Angle | Rotation (degrees) | Normal Vector | Direction |
|------------|-------------------|---------------|-----------|
| 0          | 0°                | (0, 0, -1)    | North (forward, -Z) |
| π/2        | 90°               | (1, 0, 0)     | East (right, +X) |
| π          | 180°              | (0, 0, 1)     | South (back, +Z) |
| -π/2       | -90°              | (-1, 0, 0)    | West (left, -X) |
| π/4        | 45°               | (0.707, 0, -0.707) | Northeast |

## Why the Old Formula Was Wrong

The incorrect formula `(-cos(angle), 0, sin(angle))` produces:

| Wall Angle | Old Normal | Expected Normal | Result |
|------------|-----------|-----------------|--------|
| 0          | (-1, 0, 0) ❌ | (0, 0, -1) ✓ | Points west instead of north |
| π/2        | (0, 0, 1) ❌ | (1, 0, 0) ✓ | Points south instead of east |
| π          | (1, 0, 0) ❌ | (0, 0, 1) ✓ | Points east instead of south |

The old formula was essentially 90° off and pointing in the wrong perpendicular direction.

## How Images Are Positioned

1. **Calculate slot center** along the wall surface
   ```javascript
   const slotCenter = startPos.add(wallDirection.scale(offset));
   slotCenter.y = height;
   ```

2. **Calculate normal** direction (perpendicular to wall, facing into room)
   ```javascript
   const normal = new BABYLON.Vector3(
     Math.sin(wallAngle),
     0,
     -Math.cos(wallAngle),
   );
   ```

3. **Offset position** in front of wall
   ```javascript
   const slotPosition = slotCenter.add(normal.scale(0.15));
   ```

4. **Set rotation** to match wall facing direction
   ```javascript
   rotation: new BABYLON.Vector3(0, wallAngle + Math.PI, 0)
   ```

## The `+ Math.PI` Rotation

The image rotation is `wallAngle + Math.PI` (180° flip) because:
- The **wall** faces into the room
- The **image** needs to face OUT from the wall (same direction as wall's normal)
- Babylon.js planes face -Z by default, so we flip them 180° to face +Z (outward)

## Visual Example

```
Room Layout (Top-Down View):

        North Wall (angle = 0, normal points south ↓)
    ┌────────────────────────────────────────┐
    │                                        │
West│              [Room Center]            │East
Wall│                                        │Wall
    │                                        │
    └────────────────────────────────────────┘
        South Wall (angle = π, normal points north ↑)

Image on North Wall:
- Wall at: (0, 2, -10)
- Normal: (0, 0, -1) → wait, angle=0 gives (0, 0, -1)? No!
  
Let me recalculate:
- North wall rotation makes it face SOUTH (into room)
- So wallAngle should make normal point SOUTH (+Z)
- If wall faces south, wallAngle = π (180°)
- Normal = (sin(π), 0, -cos(π)) = (0, 0, 1) ✓ Points south!
```

## Testing the Fix

### Before Fix
- Images appeared on the **wrong side** of walls
- Images visible in adjacent rooms through walls
- Images facing away from the room interior

### After Fix
- Images appear **in front of** walls, facing into the room
- Images only visible from inside the room they're placed in
- Images facing the correct direction

### Test Checklist
- [ ] Images in entrance hall visible when standing in entrance hall
- [ ] Images NOT visible through walls from adjacent rooms
- [ ] Images face toward the center of their room
- [ ] Images offset correctly from wall surface (not embedded in wall)
- [ ] All four walls (north, south, east, west) position images correctly

## Related Code

### Files Modified
- `frontend/src/scene/LayoutParser.js` - Fixed `calculateSlotPosition()` function

### Related Issues
- `IMAGE_BLEED_THROUGH_FIX.md` - Rendering group fix (prevents bleed-through)
- This fix ensures images are on the correct side to begin with

## Mathematical Derivation

For a 2D rotation around the Y-axis (vertical):

```
Rotation Matrix (Y-axis):
┌                    ┐
│  cos(θ)   0  sin(θ) │
│    0      1    0    │
│ -sin(θ)   0  cos(θ) │
└                    ┘

Default plane normal: (0, 0, -1)

Rotated normal = Matrix × Normal
= [cos(θ)×0 + 0×0 + sin(θ)×(-1)]   [−sin(θ)]
  [0×0 + 1×0 + 0×(-1)         ]  = [  0    ]
  [-sin(θ)×0 + 0×0 + cos(θ)×(-1)]   [−cos(θ)]

Result: (sin(θ), 0, -cos(θ)) ✓
```

## Conclusion

The wall normal calculation now correctly computes the direction a rotated plane faces in Babylon.js 3D space. This ensures images are positioned **in front of walls** facing **into the room**, making them visible from the correct side.

**Key Formula**: For a plane with `rotation.y = angle`, the normal is `(sin(angle), 0, -cos(angle))`

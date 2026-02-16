# Rendering Groups Solution for Z-Fighting Prevention

## Overview

This document describes the rendering-based approach to prevent z-fighting between overlapping walls at room boundaries, replacing the previous physical offset method. We use Babylon.js rendering groups to control draw order.

## Problem with Physical Offsets

### Original Approach (Deprecated)
- Entrance south walls at z=3.95
- Main north walls at z=4.0
- Main east walls at x=9.95
- Side west walls at x=10.0

### Issues
1. **Complex room definitions** - Need to track which walls touch and manually offset them
2. **Not scalable** - Each new room connection requires manual offset calculation
3. **Maintenance burden** - Adding/moving rooms requires recalculating all adjacent offsets
4. **Precision errors** - Small gaps can appear between offset walls
5. **Minimap confusion** - Physical offsets don't match logical room boundaries

## New Approach: Rendering Groups

### Concept
Instead of physically separating walls, we use Babylon.js rendering groups to control draw order. Walls at the exact same position are assigned to different rendering groups, which are rendered in sequence. This ensures consistent, flicker-free rendering regardless of viewing angle.

### Implementation

```javascript
// Assign rendering group based on room ID
const roomRenderingGroup = {
  side_exhibition: 0,  // Render first (back)
  main_gallery: 1,     // Render second (middle)
  entrance_hall: 2,    // Render last (front)
};

const renderGroup = roomRenderingGroup[roomConfig.id] || 1;
segment.renderingGroupId = renderGroup;
```

### How It Works

1. **World Position**: Walls remain at their logical positions (z=4, x=10, etc.)
2. **Rendering Groups**: Babylon.js renders groups in sequential order (0, 1, 2, ...)
3. **Rendering Order**: Lower group IDs render first (background), higher IDs render last (foreground)
4. **View-Independent**: Works from any camera angle or position
5. **Depth Testing**: Within each group, normal depth testing applies

### Rendering Group Parameters

- **renderingGroupId**: Integer value (0-3 typically used)
- **Lower values**: Render first (background layers)
- **Higher values**: Render last (foreground layers)
- **Same group**: Normal depth buffer sorting

## Room Rendering Order

```
side_exhibition:  0  (renders first - background)
main_gallery:     1  (renders second - middle)
entrance_hall:    2  (renders last - foreground)
```

This ordering ensures:
- Side exhibition walls render first (background)
- Main gallery walls render on top of side walls
- Entrance walls render last (always visible on top)
- When standing in any room, you see your room's walls correctly

## Benefits

### 1. Simple Room Definitions ✅
```json
{
  "id": "entrance_south_left",
  "start": [-6, 0, 4],
  "end": [-1.25, 0, 4]
}
```
No need to calculate offsets - walls use exact logical coordinates.

### 2. Automatic Scaling ✅
Adding new rooms only requires:
1. Define walls at logical positions
2. Add room ID to depth order map
3. No adjustment of existing walls needed

### 3. Consistent Behavior ✅
- Works from any viewing angle
- No gaps between walls
- Minimap matches physical layout
- Collision detection uses real coordinates

### 4. Easy Maintenance ✅
- Change room order by editing one object
- No recalculation of adjacent walls
- Clear separation of concerns (geometry vs rendering)

## Wall Coordinates (After Reversion)

### Entrance Hall
- North: [-6, 0, -4] to [6, 0, -4]
- South Left: [-6, 0, 4] to [-1.25, 0, 4]
- South Right: [1.25, 0, 4] to [6, 0, 4]
- East: [6, 0, -4] to [6, 0, 4]
- West: [-6, 0, -4] to [-6, 0, 4]

### Main Gallery
- North Left: [-10, 0, 4] to [-1.25, 0, 4]
- North Right: [1.25, 0, 4] to [10, 0, 4]
- South: [-10, 0, 24] to [10, 0, 24]
- East Bottom: [10, 0, 4] to [10, 0, 12.75]
- East Top: [10, 0, 15.25] to [10, 0, 24]
- West: [-10, 0, 4] to [-10, 0, 24]

### Side Exhibition
- North: [10, 0, 8] to [20, 0, 8]
- South: [10, 0, 20] to [20, 0, 20]
- East: [20, 0, 8] to [20, 0, 20]
- West Bottom: [10, 0, 8] to [10, 0, 12.75]
- West Top: [10, 0, 15.25] to [10, 0, 20]

### Doorways
- Entrance → Main: position [0, 0, 4], width 2.5
- Main → Side: position [10, 0, 14], width 2.5

All coordinates are now logical/exact with no offsets!

## Adding New Rooms

### Step 1: Define Room in JSON
```json
{
  "id": "new_exhibition",
  "name": "New Exhibition",
  "walls": [
    {
      "id": "new_north",
      "start": [20, 0, 8],
      "end": [30, 0, 8]
    }
  ]
}
```

### Step 2: Add to Rendering Group Order
```javascript
const roomRenderingGroup = {
  new_exhibition: 0,    // ← Add here (if it's furthest back)
  side_exhibition: 1,   // Or adjust existing order
  main_gallery: 2,
  entrance_hall: 3,
};
```

### Step 3: Done!
No need to adjust any other walls or coordinates.

## Performance Considerations

### GPU Support
- **Rendering groups** are a Babylon.js feature built on standard draw call ordering
- Supported by all browsers that support Babylon.js
- Minimal performance impact (just reorders draw calls)
- More reliable than depth bias for this use case

### Group Precision
- Up to 4 rendering groups available (0-3)
- Perfect for most gallery layouts
- No depth precision issues
- Completely eliminates z-fighting

### Best Practices
1. Use **integer group IDs** (0, 1, 2, 3)
2. Keep groups **sequential** (no gaps in numbering)
3. Order rooms **logically** (furthest → nearest, or entrance → deeper areas)
4. Test from **multiple angles** to verify no flickering
5. Reserve group 0 for backgrounds, higher groups for foreground

## Comparison

| Aspect | Physical Offsets | Rendering Groups |
|--------|-----------------|------------------|
| Room Definition | Complex (manual offsets) | Simple (exact coords) |
| Scalability | Poor (recalc each time) | Excellent (one line) |
| Maintenance | High (track dependencies) | Low (edit one map) |
| Minimap Accuracy | Slightly off | Perfect match |
| Collision Detection | Slightly off | Exact |
| Visual Quality | Perfect | Perfect |
| Performance | Same | Minimal overhead |
| Browser Support | Universal | Universal (Babylon.js) |
| Reliability | Good | Excellent |

## Technical Details

### Babylon.js Rendering Groups

```javascript
mesh.renderingGroupId = 0;  // Background layer
mesh.renderingGroupId = 1;  // Middle layer
mesh.renderingGroupId = 2;  // Foreground layer
```

Rendering groups are processed in order during the render loop:

```javascript
// Pseudo-code for rendering
for (let groupId = 0; groupId < maxGroups; groupId++) {
  const meshes = getMeshesInGroup(groupId);
  for (const mesh of meshes) {
    renderMesh(mesh); // Normal depth testing within group
  }
}
```

### How Rendering Groups Work

1. **Scene groups meshes** by renderingGroupId
2. **Groups render sequentially** (0, then 1, then 2, etc.)
3. **Within each group**, normal depth testing applies
4. **Later groups always appear** in front of earlier groups
5. **Transparent meshes** can use higher groups for proper sorting

### Group ID Values

- **0**: Background/furthest layers
- **1**: Middle layers
- **2**: Foreground layers
- **3**: Overlay/UI elements (rarely used for 3D geometry)

## Troubleshooting

### Still Seeing Flicker?
1. Verify room IDs match the rendering group map
2. Check for typos in room ID strings
3. Ensure group IDs are sequential (0, 1, 2...)
4. Clear browser cache and rebuild

### Walls Rendering in Wrong Order?
1. Verify group order logic (lower = back, higher = front)
2. Check if room IDs are correct in JSON
3. Ensure all rooms have an entry in rendering group map
4. Test from different camera positions to verify consistency

### Gaps Between Walls?
This shouldn't happen with depth bias - walls remain at exact positions. If you see gaps:
1. Check wall coordinates in JSON
2. Verify walls actually meet (check endpoints)
3. Ensure no accidental physical offsets remain

## Future Enhancements

### Dynamic Group Assignment
Instead of manual mapping, calculate rendering groups based on room connectivity:

```javascript
function calculateRoomRenderGroup(roomId, visited = new Set()) {
  if (visited.has(roomId)) return 0;
  visited.add(roomId);
  
  const connections = getRoomConnections(roomId);
  const groups = connections.map(r => calculateRoomRenderGroup(r, visited));
  return Math.min(3, Math.max(0, ...groups) + 1);
}
```

### Additional Optimization
For very complex galleries, consider:

1. **Layer-based grouping**: Group multiple distant rooms together
2. **Dynamic switching**: Change groups based on camera position
3. **Culling**: Only render rooms visible from current position

## Conclusion

The rendering groups approach provides a **clean, scalable, and maintainable** solution for preventing z-fighting at room boundaries. It eliminates the complexity of physical offsets while maintaining perfect visual quality and performance.

**Key Takeaway**: Solve rendering problems with rendering solutions, not geometric workarounds. Rendering groups provide deterministic draw order that works from any viewing angle.

## Related Files

- `crates/3d-gallery/frontend/src/scene/LayoutParser.js` - Rendering groups implementation
- `crates/3d-gallery/frontend/src/layouts/demo-gallery.json` - Room definitions (no offsets)
- `crates/3d-gallery/WALL_SPLITTING_FIX.md` - Previous offset approach (deprecated)
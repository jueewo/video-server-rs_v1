import { useEffect, useRef } from "preact/hooks";

/**
 * Minimap component - Shows top-down view of gallery with camera position
 *
 * The minimap uses a flipped Z axis (+Z = UP on canvas) so that:
 * - Forward (camera facing +Z) points UP on the minimap
 * - Turning right (D key, rotation.y increases) rotates the arrow clockwise
 * - The arrow always matches the actual movement direction
 *
 * This compensates for Babylon.js's left-handed coordinate system where
 * positive Y rotation is counterclockwise when viewed from above.
 */
export function Minimap({ camera, gallery }) {
  const canvasRef = useRef(null);

  useEffect(() => {
    if (!camera || !canvasRef.current || !gallery) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");

    // Calculate bounding box for all rooms
    let minX = Infinity,
      maxX = -Infinity;
    let minZ = Infinity,
      maxZ = -Infinity;

    gallery.rooms.forEach((room) => {
      const { position, dimensions } = room;
      const x = position.x;
      const z = position.z;
      const halfWidth = dimensions.width / 2;
      const halfDepth = dimensions.depth / 2;

      minX = Math.min(minX, x - halfWidth);
      maxX = Math.max(maxX, x + halfWidth);
      minZ = Math.min(minZ, z - halfDepth);
      maxZ = Math.max(maxZ, z + halfDepth);
    });

    const totalWidth = maxX - minX;
    const totalDepth = maxZ - minZ;
    const padding = 20;
    const availableWidth = canvas.width - padding * 2;
    const availableHeight = canvas.height - padding * 2 - 40; // Leave space for text

    const scaleX = availableWidth / totalWidth;
    const scaleZ = availableHeight / totalDepth;
    const scale = Math.min(scaleX, scaleZ);

    // Helper: map world X to canvas X (unchanged)
    const toCanvasX = (worldX) => (worldX - minX) * scale + padding;

    // Helper: map world Z to canvas Y (FLIPPED: +Z = UP on canvas)
    const toCanvasY = (worldZ) => (maxZ - worldZ) * scale + padding;

    // Update minimap at 30fps
    const updateInterval = setInterval(() => {
      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw all rooms
      gallery.rooms.forEach((room) => {
        const { position, dimensions, name } = room;

        // Room top-left corner in canvas space (flipped Z)
        const roomX = toCanvasX(position.x - dimensions.width / 2);
        const roomY = toCanvasY(position.z + dimensions.depth / 2);
        const roomWidth = dimensions.width * scale;
        const roomDepth = dimensions.depth * scale;

        // Draw room floor
        ctx.fillStyle = "rgba(100, 100, 100, 0.3)";
        ctx.fillRect(roomX, roomY, roomWidth, roomDepth);

        // Draw room outline
        ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
        ctx.lineWidth = 2;
        ctx.strokeRect(roomX, roomY, roomWidth, roomDepth);

        // Draw room name
        ctx.fillStyle = "rgba(255, 255, 255, 0.6)";
        ctx.font = "9px monospace";
        ctx.fillText(name, roomX + 5, roomY + 15);
      });

      // Draw doorways - orientation matches the wall direction
      gallery.doorways.forEach((doorway) => {
        if (doorway && doorway.width) {
          const doorX = toCanvasX(doorway.position.x);
          const doorY = toCanvasY(doorway.position.z);
          const doorWidth = doorway.width * scale;

          // Find which wall this doorway belongs to (closest match)
          let isHorizontal = true; // default: wall runs along X axis
          let closestDist = Infinity;

          gallery.walls.forEach((wall) => {
            if (wall.startPos && wall.endPos) {
              const wallDX = Math.abs(wall.endPos.x - wall.startPos.x);
              const wallDZ = Math.abs(wall.endPos.z - wall.startPos.z);

              // Check if doorway is close to this wall's start position
              const distX = Math.abs(doorway.position.x - wall.startPos.x);
              const distZ = Math.abs(doorway.position.z - wall.startPos.z);
              const totalDist = Math.min(distX, distZ);

              if (totalDist < 2 && totalDist < closestDist) {
                // This is the closest matching wall
                closestDist = totalDist;
                // Wall orientation: if wall runs along X, door is horizontal; if along Z, door is vertical
                isHorizontal = wallDX > wallDZ;
              }
            }
          });

          // Draw doorway along the same direction as the wall
          ctx.fillStyle = "rgba(100, 255, 100, 0.7)";
          if (isHorizontal) {
            ctx.fillRect(doorX - doorWidth / 2, doorY - 2, doorWidth, 4);
          } else {
            ctx.fillRect(doorX - 2, doorY - doorWidth / 2, 4, doorWidth);
          }

          // Add border for better visibility
          ctx.strokeStyle = "rgba(150, 255, 150, 0.9)";
          ctx.lineWidth = 1;
          if (isHorizontal) {
            ctx.strokeRect(doorX - doorWidth / 2, doorY - 2, doorWidth, 4);
          } else {
            ctx.strokeRect(doorX - 2, doorY - doorWidth / 2, 4, doorWidth);
          }
        }
      });

      // Draw camera position (flipped Z)
      const camX = toCanvasX(camera.position.x);
      const camY = toCanvasY(camera.position.z);

      // Camera direction indicator
      // Babylon.js left-handed: forward = (sin(rotY), 0, cos(rotY))
      // On flipped minimap (+Z = UP): canvasDir = (sin(rotY), -cos(rotY))
      // This gives: rotY=0 → arrow points UP, D key (rotY increases) → clockwise rotation
      const rotY = camera.rotation.y;
      const dirLength = 12;
      const dirX = Math.sin(rotY) * dirLength;
      const dirY = -Math.cos(rotY) * dirLength;

      // Perpendicular vector for view cone (rotate 90°)
      const perpX = -dirY * 0.6;
      const perpY = dirX * 0.6;

      // Draw view cone
      ctx.fillStyle = "rgba(59, 130, 246, 0.3)";
      ctx.beginPath();
      ctx.moveTo(camX, camY);
      ctx.lineTo(camX + dirX + perpX, camY + dirY + perpY);
      ctx.lineTo(camX + dirX - perpX, camY + dirY - perpY);
      ctx.closePath();
      ctx.fill();

      // Draw camera dot
      ctx.fillStyle = "#3b82f6";
      ctx.strokeStyle = "white";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(camX, camY, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      // Draw direction line
      ctx.strokeStyle = "white";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(camX, camY);
      ctx.lineTo(camX + dirX, camY + dirY);
      ctx.stroke();

      // Draw coordinates text
      ctx.fillStyle = "rgba(255, 255, 255, 0.8)";
      ctx.font = "10px monospace";
      ctx.fillText(
        `X: ${camera.position.x.toFixed(1)} Z: ${camera.position.z.toFixed(1)}`,
        padding,
        canvas.height - 10,
      );
    }, 33); // ~30fps

    return () => clearInterval(updateInterval);
  }, [camera, gallery]);

  return (
    <div
      style={{
        position: "fixed",
        bottom: "20px",
        right: "20px",
        background: "rgba(0, 0, 0, 0.7)",
        backdropFilter: "blur(10px)",
        borderRadius: "12px",
        padding: "15px",
        border: "1px solid rgba(255, 255, 255, 0.2)",
        boxShadow: "0 4px 20px rgba(0, 0, 0, 0.5)",
        zIndex: 100,
      }}
    >
      <div
        style={{
          color: "white",
          fontSize: "12px",
          fontWeight: "bold",
          marginBottom: "8px",
          textAlign: "center",
        }}
      >
        MINIMAP
      </div>
      <canvas
        ref={canvasRef}
        width="180"
        height="200"
        style={{
          display: "block",
          imageRendering: "crisp-edges",
        }}
      />
      <div
        style={{
          marginTop: "8px",
          fontSize: "10px",
          color: "rgba(255, 255, 255, 0.7)",
          textAlign: "center",
        }}
      >
        WASD to move
      </div>
    </div>
  );
}

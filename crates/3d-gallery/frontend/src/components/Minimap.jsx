import { useEffect, useRef } from "preact/hooks";

/**
 * Minimap component - Shows top-down view of gallery with camera position
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

    // Update minimap at 30fps
    const updateInterval = setInterval(() => {
      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw all rooms
      gallery.rooms.forEach((room) => {
        const { position, dimensions, name } = room;
        const roomX =
          (position.x - dimensions.width / 2 - minX) * scale + padding;
        const roomZ =
          (position.z - dimensions.depth / 2 - minZ) * scale + padding;
        const roomWidth = dimensions.width * scale;
        const roomDepth = dimensions.depth * scale;

        // Draw room floor
        ctx.fillStyle = "rgba(100, 100, 100, 0.3)";
        ctx.fillRect(roomX, roomZ, roomWidth, roomDepth);

        // Draw room outline
        ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
        ctx.lineWidth = 2;
        ctx.strokeRect(roomX, roomZ, roomWidth, roomDepth);

        // Draw room name
        ctx.fillStyle = "rgba(255, 255, 255, 0.6)";
        ctx.font = "9px monospace";
        ctx.fillText(name, roomX + 5, roomZ + 15);
      });

      // Draw doorways
      gallery.doorways.forEach((doorway) => {
        if (doorway) {
          const doorX = (doorway.position.x - minX) * scale + padding;
          const doorZ = (doorway.position.z - minZ) * scale + padding;
          ctx.fillStyle = "rgba(100, 255, 100, 0.5)";
          ctx.fillRect(doorX - 2, doorZ - 2, 4, 4);
        }
      });

      // Draw camera position
      const camX = (camera.position.x - minX) * scale + padding;
      const camZ = (camera.position.z - minZ) * scale + padding;

      // Camera direction indicator
      const camRotY = camera.rotation.y + Math.PI;
      const dirLength = 12;
      const dirX = Math.sin(camRotY) * dirLength;
      const dirZ = -Math.cos(camRotY) * dirLength;

      // Draw view cone
      ctx.fillStyle = "rgba(59, 130, 246, 0.3)";
      ctx.beginPath();
      ctx.moveTo(camX, camZ);
      ctx.lineTo(camX + dirX + 8, camZ + dirZ - 8);
      ctx.lineTo(camX + dirX - 8, camZ + dirZ + 8);
      ctx.closePath();
      ctx.fill();

      // Draw camera dot
      ctx.fillStyle = "#3b82f6";
      ctx.strokeStyle = "white";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(camX, camZ, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      // Draw direction line
      ctx.strokeStyle = "white";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(camX, camZ);
      ctx.lineTo(camX + dirX, camZ + dirZ);
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

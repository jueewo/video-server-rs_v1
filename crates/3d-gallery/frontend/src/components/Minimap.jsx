import { useEffect, useRef } from "preact/hooks";

/**
 * Minimap component - Shows top-down view of gallery with camera position
 */
export function Minimap({ camera, roomWidth = 20, roomDepth = 20 }) {
  const canvasRef = useRef(null);

  useEffect(() => {
    if (!camera || !canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    const scale = 8; // Scale factor for map
    const mapWidth = roomWidth * scale;
    const mapDepth = roomDepth * scale;

    // Update minimap at 30fps
    const updateInterval = setInterval(() => {
      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw room outline
      ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
      ctx.lineWidth = 2;
      ctx.strokeRect(10, 10, mapWidth, mapDepth);

      // Draw walls
      ctx.strokeStyle = "rgba(255, 255, 255, 0.3)";
      ctx.lineWidth = 1;

      // North wall (top)
      ctx.beginPath();
      ctx.moveTo(10, 10);
      ctx.lineTo(10 + mapWidth, 10);
      ctx.stroke();

      // South wall (bottom)
      ctx.beginPath();
      ctx.moveTo(10, 10 + mapDepth);
      ctx.lineTo(10 + mapWidth, 10 + mapDepth);
      ctx.stroke();

      // East wall (right)
      ctx.beginPath();
      ctx.moveTo(10 + mapWidth, 10);
      ctx.lineTo(10 + mapWidth, 10 + mapDepth);
      ctx.stroke();

      // West wall (left)
      ctx.beginPath();
      ctx.moveTo(10, 10);
      ctx.lineTo(10, 10 + mapDepth);
      ctx.stroke();

      // Draw camera position
      const camX = ((camera.position.x + roomWidth / 2) / roomWidth) * mapWidth;
      const camZ = ((camera.position.z + roomDepth / 2) / roomDepth) * mapDepth;

      // Camera direction indicator
      const camRotY = camera.rotation.y;
      const dirLength = 15;
      const dirX = Math.sin(camRotY) * dirLength;
      const dirZ = Math.cos(camRotY) * dirLength;

      // Draw view cone
      ctx.fillStyle = "rgba(59, 130, 246, 0.2)";
      ctx.beginPath();
      ctx.moveTo(10 + camX, 10 + camZ);
      ctx.lineTo(10 + camX + dirX + 10, 10 + camZ + dirZ - 10);
      ctx.lineTo(10 + camX + dirX - 10, 10 + camZ + dirZ + 10);
      ctx.closePath();
      ctx.fill();

      // Draw camera dot
      ctx.fillStyle = "#3b82f6";
      ctx.strokeStyle = "white";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(10 + camX, 10 + camZ, 6, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      // Draw direction line
      ctx.strokeStyle = "white";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(10 + camX, 10 + camZ);
      ctx.lineTo(10 + camX + dirX, 10 + camZ + dirZ);
      ctx.stroke();

      // Draw coordinates text
      ctx.fillStyle = "rgba(255, 255, 255, 0.8)";
      ctx.font = "10px monospace";
      ctx.fillText(
        `X: ${camera.position.x.toFixed(1)} Z: ${camera.position.z.toFixed(1)}`,
        10,
        mapDepth + 30,
      );
    }, 33); // ~30fps

    return () => clearInterval(updateInterval);
  }, [camera, roomWidth, roomDepth]);

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

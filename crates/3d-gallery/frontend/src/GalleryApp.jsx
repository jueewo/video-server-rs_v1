import { useEffect, useRef, useState } from "preact/hooks";
import * as BABYLON from "@babylonjs/core";
import { fetchGalleryData } from "./api/galleryApi";
import {
  createGalleryFromLayout,
  mapMediaToSlots,
  disposeGallery,
} from "./scene/LayoutParser";
import { createImageFrame } from "./scene/ImageFrame";
import { createVideoScreen } from "./scene/VideoScreen";
import { VideoPlayer } from "./components/VideoPlayer";
import { Minimap } from "./components/Minimap";
import demoLayout from "./layouts/demo-gallery.json";

export default function GalleryApp({
  accessCode,
  apiEndpoint,
  onReady,
  onError,
}) {
  const canvasRef = useRef(null);
  const cameraRef = useRef(null);
  const sceneRef = useRef(null);
  const [loading, setLoading] = useState(true);
  const [galleryData, setGalleryData] = useState(null);
  const [selectedImage, setSelectedImage] = useState(null);
  const [cameraReady, setCameraReady] = useState(false);
  const [minimapVisible, setMinimapVisible] = useState(true);
  const [helpVisible, setHelpVisible] = useState(false);

  // Fetch gallery data on mount
  useEffect(() => {
    async function loadGalleryData() {
      try {
        const data = await fetchGalleryData(accessCode, apiEndpoint);
        setGalleryData(data);
        setLoading(false);
      } catch (error) {
        console.error("Failed to load gallery data:", error);
        if (onError) {
          onError(
            "Failed to Load Gallery",
            error.message ||
              "Unable to load gallery data. Please check your access code and try again.",
          );
        }
        setLoading(false);
      }
    }

    loadGalleryData();
  }, [accessCode, apiEndpoint, onError]);

  // Initialize Babylon.js scene - only once when data is loaded
  useEffect(() => {
    if (loading || !canvasRef.current || !galleryData) {
      return;
    }

    console.log("Initializing Babylon.js gallery scene...");
    console.log("Gallery data:", galleryData);

    // Create engine
    const engine = new BABYLON.Engine(canvasRef.current, true, {
      preserveDrawingBuffer: true,
      stencil: true,
    });

    // Create scene
    const scene = new BABYLON.Scene(engine);
    scene.clearColor = new BABYLON.Color4(0.1, 0.1, 0.15, 1.0);

    // Store scene ref for pointer event control
    sceneRef.current = scene;

    // Create camera - FPS style camera for walking around
    const camera = new BABYLON.UniversalCamera(
      "camera",
      new BABYLON.Vector3(0, 1.8, -5), // Start position - eye level, facing north
      scene,
    );
    camera.setTarget(new BABYLON.Vector3(0, 1.8, 0)); // Look at center
    camera.attachControl(canvasRef.current, true);

    // Camera movement settings
    camera.speed = 0.3;
    camera.angularSensibility = 2000;
    camera.keysUp = []; // Disable default WASD
    camera.keysDown = [];
    camera.keysLeft = [];
    camera.keysRight = [];

    // Limit vertical look angle
    camera.upperBetaLimit = Math.PI / 2 - 0.1;
    camera.lowerBetaLimit = 0.1;

    // Store camera ref for later control detachment
    cameraRef.current = camera;
    setCameraReady(true); // Trigger re-render to show minimap

    // Fix aspect ratio
    camera.mode = BABYLON.Camera.PERSPECTIVE_CAMERA;
    const updateAspectRatio = () => {
      const canvas = canvasRef.current;
      if (canvas) {
        camera.getEngine().resize();
      }
    };
    updateAspectRatio();

    // Create gallery from JSON layout
    console.log("Loading gallery layout...");
    const gallery = createGalleryFromLayout(scene, demoLayout);

    // Set camera spawn point from layout
    if (demoLayout.spawn_point) {
      const spawn = demoLayout.spawn_point;
      camera.position = new BABYLON.Vector3(...spawn.position);
      camera.rotation = new BABYLON.Vector3(...spawn.rotation);
    }

    // Store ceiling references for dynamic transparency (first room's ceiling)
    const ceiling = gallery.rooms[0]?.ceiling;
    const ceilingHeight = gallery.rooms[0]?.dimensions.height || 4;

    // Prepare media items
    const mediaItems = galleryData.items.map((item) => ({
      id: item.id,
      title: item.title || "Untitled",
      description: item.description || "",
      url: item.url,
      thumbnail_url: item.thumbnail_url || item.url,
      width: item.width,
      height: item.height,
      tags: item.tags || [],
      duration: item.duration,
      media_type: item.media_type,
      type: item.media_type,
    }));

    // Map media to slots using the layout's media_mapping
    const mappedSlots = mapMediaToSlots(
      gallery,
      demoLayout.media_mapping,
      mediaItems,
    );

    console.log(`Mapped ${mappedSlots.length} media items to slots`);

    // Create frames and screens for mapped slots
    const frames = [];
    const videoScreens = [];

    mappedSlots.forEach(({ slot, media }) => {
      if (media.type === "image") {
        const frame = createImageFrame(scene, media, {
          position: slot.position,
          rotation: slot.rotation,
          width: slot.width,
          frameThickness: 0.12,
        });

        // Setup click interaction
        if (frame.framePlane) {
          frame.framePlane.actionManager = new BABYLON.ActionManager(scene);
          frame.framePlane.actionManager.registerAction(
            new BABYLON.ExecuteCodeAction(
              BABYLON.ActionManager.OnPickDownTrigger,
              () => {
                console.log("Image clicked:", media.title);
                setSelectedImage(media);
              },
            ),
          );
        }

        frames.push(frame);
      } else if (media.type === "video") {
        const screen = createVideoScreen(scene, media, {
          position: slot.position,
          rotation: slot.rotation,
          width: slot.width,
          aspectRatio: 16 / 9,
          frameThickness: 0.15,
          autoPlay: false,
        });

        // Setup click interaction
        if (screen.screenPlane) {
          screen.screenPlane.actionManager = new BABYLON.ActionManager(scene);
          screen.screenPlane.actionManager.registerAction(
            new BABYLON.ExecuteCodeAction(
              BABYLON.ActionManager.OnPickDownTrigger,
              () => {
                console.log("Video clicked:", media.title);
                setSelectedImage(media);
              },
            ),
          );
        }

        videoScreens.push(screen);
      }
    });

    console.log(
      `Created ${frames.length} frames and ${videoScreens.length} video screens`,
    );

    // Store for cleanup
    const allFrames = frames;
    const allVideoScreens = videoScreens;

    // Setup scene pointer observable to handle clicks before camera
    scene.onPointerObservable.add((pointerInfo) => {
      if (pointerInfo.type === BABYLON.PointerEventTypes.POINTERDOWN) {
        const pickResult = pointerInfo.pickInfo;
        if (pickResult.hit && pickResult.pickedMesh) {
          const metadata = pickResult.pickedMesh.metadata;
          if (
            metadata &&
            (metadata.type === "image" || metadata.type === "video")
          ) {
            console.log("Media picked via observable:", metadata.title);
            // Prevent camera from handling this click
            pointerInfo.event.preventDefault();
            pointerInfo.event.stopPropagation();
            setSelectedImage(metadata);
          }
        }
      }
    });

    // Enable proper clearing
    scene.autoClear = true;
    scene.autoClearDepthAndStencil = true;

    // Enable frustum culling for better performance
    scene.frustumCullingEnabled = true;

    // Start render loop with dynamic ceiling transparency and frustum culling
    engine.runRenderLoop(() => {
      // Check camera height and adjust ceiling transparency
      if (camera && ceiling) {
        const cameraHeight = camera.position.y;

        if (cameraHeight > ceilingHeight) {
          // Above ceiling - make it semi-transparent
          if (ceiling.material.alpha !== 0.3) {
            ceiling.material.alpha = 0.3;
            ceiling.material.needAlphaBlending = () => true;
          }
        } else {
          // Below ceiling - make it opaque
          if (ceiling.material.alpha !== 1.0) {
            ceiling.material.alpha = 1.0;
            ceiling.material.needAlphaBlending = () => false;
          }
        }
      }

      // Frustum culling - hide objects not in camera view
      if (camera && scene.frustumPlanes) {
        // Update all image frames
        frames.forEach((frame) => {
          if (frame.framePlane) {
            frame.framePlane.isVisible = frame.framePlane.isInFrustum(
              scene.frustumPlanes,
            );
          }
        });

        // Update all video screens
        allVideoScreens.forEach((screen) => {
          const inFrustum =
            screen.screenPlane &&
            screen.screenPlane.isInFrustum(scene.frustumPlanes);

          if (screen.screenPlane) {
            screen.screenPlane.isVisible = inFrustum;
          }

          // Update overlays based on frustum AND video state
          if (screen.playButtonOverlay) {
            const shouldShow =
              inFrustum &&
              (screen.videoElement.paused || screen.videoElement.ended);
            screen.playButtonOverlay.isVisible = shouldShow;
          }

          if (screen.progressBar && screen.progressBar.plane) {
            const shouldShow =
              inFrustum &&
              !screen.videoElement.paused &&
              !screen.videoElement.ended;
            screen.progressBar.plane.isVisible = shouldShow;
          }
        });
      }

      scene.render();
    });

    // Handle window resize - critical for maintaining aspect ratio
    const handleResize = () => {
      engine.resize(true); // Force resize with aspect ratio recalculation
    };
    window.addEventListener("resize", handleResize);

    // Initial resize to ensure correct aspect ratio
    setTimeout(() => engine.resize(true), 100);

    // Notify that scene is ready
    if (onReady) {
      onReady();
    }

    console.log("Gallery scene initialized successfully!");
    console.log(
      `Gallery contains ${allFrames.length} images and ${allVideoScreens.length} videos`,
    );

    // Cleanup function
    return () => {
      console.log("Cleaning up Babylon.js scene...");
      window.removeEventListener("resize", handleResize);
      engine.stopRenderLoop();

      // Pause all videos first
      allVideoScreens.forEach((screen) => {
        if (screen.videoElement && !screen.videoElement.paused) {
          screen.videoElement.pause();
        }
        if (screen.hls) {
          screen.hls.destroy();
        }
      });

      // Dispose frames
      allFrames.forEach((frame) => {
        if (frame.framePlane) frame.framePlane.dispose();
        if (frame.frameBorder) {
          frame.frameBorder.forEach((piece) => piece.dispose());
        }
      });

      // Dispose video screens
      allVideoScreens.forEach((screen) => {
        if (screen.parent) screen.parent.dispose();
        if (screen.screenPlane) screen.screenPlane.dispose();
      });

      // Dispose gallery
      disposeGallery(gallery);

      // Dispose scene and engine
      scene.dispose();
      engine.dispose();
    };
  }, [loading, galleryData, onReady]);

  // Handle WASD keyboard movement
  useEffect(() => {
    if (!cameraRef.current) return;

    const camera = cameraRef.current;
    const moveSpeed = 0.3;
    const keysPressed = {};

    const handleKeyDown = (event) => {
      // H key to toggle help panel
      if (event.key === "h" || event.key === "H") {
        setHelpVisible((prev) => !prev);
        return;
      }

      // M key to toggle minimap
      if (event.key === "m" || event.key === "M") {
        setMinimapVisible((prev) => !prev);
        return;
      }

      // ESC key to close overlay
      if (event.key === "Escape" && selectedImage) {
        setSelectedImage(null);
        return;
      }

      // Don't handle movement when overlay is open
      if (selectedImage) return;

      // Map arrow keys to WASD
      const key = event.key.toLowerCase();
      if (key === "arrowup") {
        keysPressed["w"] = true;
      } else if (key === "arrowdown") {
        keysPressed["s"] = true;
      } else if (key === "arrowleft") {
        keysPressed["a"] = true;
      } else if (key === "arrowright") {
        keysPressed["d"] = true;
      } else {
        keysPressed[key] = true;
      }
    };

    const handleKeyUp = (event) => {
      const key = event.key.toLowerCase();
      if (key === "arrowup") {
        keysPressed["w"] = false;
      } else if (key === "arrowdown") {
        keysPressed["s"] = false;
      } else if (key === "arrowleft") {
        keysPressed["a"] = false;
      } else if (key === "arrowright") {
        keysPressed["d"] = false;
      } else {
        keysPressed[key] = false;
      }
    };

    // Gamepad support
    let gamepadIndex = null;
    const checkGamepad = () => {
      const gamepads = navigator.getGamepads();
      for (let i = 0; i < gamepads.length; i++) {
        if (gamepads[i]) {
          gamepadIndex = i;
          break;
        }
      }
    };

    window.addEventListener("gamepadconnected", (e) => {
      console.log("Gamepad connected:", e.gamepad.id);
      gamepadIndex = e.gamepad.index;
    });

    window.addEventListener("gamepaddisconnected", (e) => {
      console.log("Gamepad disconnected");
      if (gamepadIndex === e.gamepad.index) {
        gamepadIndex = null;
      }
    });

    checkGamepad();

    // Movement loop
    const moveInterval = setInterval(() => {
      if (selectedImage) return; // Don't move when overlay is open

      let moved = false;

      // Get camera forward direction (already normalized)
      const forward = camera.getDirection(BABYLON.Axis.Z);
      const right = camera.getDirection(BABYLON.Axis.X);

      // Check gamepad input
      let gamepadForward = 0;
      let gamepadStrafe = 0;
      let gamepadTurn = 0;
      if (gamepadIndex !== null) {
        const gamepad = navigator.getGamepads()[gamepadIndex];
        if (gamepad) {
          // Left stick: movement (axis 0 = horizontal, axis 1 = vertical)
          const leftStickX =
            Math.abs(gamepad.axes[0]) > 0.1 ? gamepad.axes[0] : 0;
          const leftStickY =
            Math.abs(gamepad.axes[1]) > 0.1 ? gamepad.axes[1] : 0;

          // Right stick: look/turn (axis 2 = horizontal)
          const rightStickX =
            Math.abs(gamepad.axes[2]) > 0.1 ? gamepad.axes[2] : 0;

          gamepadForward = -leftStickY; // Inverted
          gamepadStrafe = leftStickX;
          gamepadTurn = rightStickX;
        }
      }

      // W/S or Arrow Up/Down or Gamepad - Move forward/backward
      if (keysPressed["w"] || gamepadForward > 0) {
        camera.position.addInPlace(
          forward.scale(moveSpeed * (gamepadForward || 1)),
        );
        moved = true;
      }
      if (keysPressed["s"] || gamepadForward < 0) {
        camera.position.addInPlace(
          forward.scale(-moveSpeed * (Math.abs(gamepadForward) || 1)),
        );
        moved = true;
      }

      // A/D or Arrow Left/Right or Gamepad - Turn left/right
      if (keysPressed["a"] || gamepadTurn < 0) {
        camera.rotation.y -= 0.05 * (Math.abs(gamepadTurn) || 1); // Turn left
        moved = true;
      }
      if (keysPressed["d"] || gamepadTurn > 0) {
        camera.rotation.y += 0.05 * (gamepadTurn || 1); // Turn right
        moved = true;
      }

      // Gamepad strafe (left stick horizontal)
      if (Math.abs(gamepadStrafe) > 0) {
        camera.position.addInPlace(right.scale(moveSpeed * gamepadStrafe));
        moved = true;
      }

      // Clamp camera position to stay inside gallery
      if (moved) {
        camera.position.x = Math.max(-9, Math.min(9, camera.position.x));
        camera.position.z = Math.max(-9, Math.min(9, camera.position.z));
        camera.position.y = Math.max(1.5, Math.min(3, camera.position.y)); // Keep at eye level
      }
    }, 16); // ~60fps

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      clearInterval(moveInterval);
    };
  }, [selectedImage, cameraReady]);

  // Handle camera controls based on overlay state
  useEffect(() => {
    if (!cameraRef.current || !canvasRef.current) return;

    if (selectedImage) {
      // Detach camera when overlay opens
      cameraRef.current.detachControl(canvasRef.current);
      console.log("Camera detached - overlay open");
    } else {
      // Reattach camera when overlay closes
      const timeout = setTimeout(() => {
        if (cameraRef.current && canvasRef.current) {
          if (document.pointerLockElement) {
            document.exitPointerLock();
          }
          cameraRef.current.attachControl(canvasRef.current, true);
          console.log("Camera reattached - overlay closed");
        }
      }, 100);

      return () => clearTimeout(timeout);
    }
  }, [selectedImage]);

  if (loading) {
    return null; // Loading screen is handled by template
  }

  return (
    <div style={{ width: "100%", height: "100%" }}>
      <canvas
        ref={canvasRef}
        style={{
          width: "100%",
          height: "100%",
          display: "block",
          outline: "none",
        }}
      />

      {/* Minimap */}
      {!loading &&
        cameraReady &&
        cameraRef.current &&
        !selectedImage &&
        minimapVisible && (
          <Minimap camera={cameraRef.current} roomWidth={30} roomDepth={30} />
        )}

      {/* Help Panel */}
      {helpVisible && (
        <div
          style={{
            position: "fixed",
            top: "20px",
            left: "20px",
            background: "rgba(0, 0, 0, 0.85)",
            backdropFilter: "blur(10px)",
            borderRadius: "12px",
            padding: "20px",
            border: "1px solid rgba(255, 255, 255, 0.2)",
            boxShadow: "0 4px 20px rgba(0, 0, 0, 0.5)",
            zIndex: 100,
            color: "white",
            maxWidth: "300px",
          }}
        >
          <div
            style={{
              fontSize: "16px",
              fontWeight: "bold",
              marginBottom: "15px",
              textAlign: "center",
              borderBottom: "1px solid rgba(255, 255, 255, 0.2)",
              paddingBottom: "10px",
            }}
          >
            🎮 CONTROLS
          </div>
          <div style={{ fontSize: "13px", lineHeight: "1.8" }}>
            <div style={{ marginBottom: "10px" }}>
              <strong>Movement:</strong>
            </div>
            <div style={{ marginLeft: "10px", marginBottom: "15px" }}>
              <div>W / ↑ - Move forward</div>
              <div>S / ↓ - Move backward</div>
              <div>A / ← - Turn left</div>
              <div>D / → - Turn right</div>
              <div style={{ fontSize: "11px", opacity: 0.7, marginTop: "5px" }}>
                🎮 Gamepad supported
              </div>
            </div>
            <div style={{ marginBottom: "10px" }}>
              <strong>View:</strong>
            </div>
            <div style={{ marginLeft: "10px", marginBottom: "15px" }}>
              <div>Mouse - Look around</div>
              <div>Scroll - Zoom in/out</div>
            </div>
            <div style={{ marginBottom: "10px" }}>
              <strong>Interaction:</strong>
            </div>
            <div style={{ marginLeft: "10px", marginBottom: "15px" }}>
              <div>Click - View media</div>
              <div>Hover - Preview video</div>
            </div>
            <div style={{ marginBottom: "10px" }}>
              <strong>Toggles:</strong>
            </div>
            <div style={{ marginLeft: "10px" }}>
              <div>M - Toggle minimap</div>
              <div>H - Toggle this help</div>
              <div>ESC - Close overlay</div>
            </div>
          </div>
        </div>
      )}

      {/* Image overlay when clicked */}
      {selectedImage && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            width: "100%",
            height: "100%",
            background: "rgba(0, 0, 0, 0.9)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
            padding: "20px",
            cursor: "pointer",
          }}
          onClick={(e) => {
            e.stopPropagation();
            setSelectedImage(null);
          }}
        >
          <div
            style={{
              maxWidth: "90%",
              maxHeight: "90%",
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              cursor: "default",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            {selectedImage.type === "video" ? (
              <VideoPlayer url={selectedImage.url} autoPlay={true} />
            ) : (
              <img
                src={selectedImage.url}
                alt={selectedImage.title}
                style={{
                  maxWidth: "100%",
                  maxHeight: "80vh",
                  objectFit: "contain",
                  borderRadius: "8px",
                  boxShadow: "0 4px 20px rgba(0, 0, 0, 0.5)",
                }}
              />
            )}
            <div
              style={{
                marginTop: "20px",
                textAlign: "center",
                color: "white",
                maxWidth: "600px",
              }}
            >
              <h2 style={{ margin: "0 0 10px 0", fontSize: "24px" }}>
                {selectedImage.title}
              </h2>
              {selectedImage.description && (
                <p style={{ margin: "0", fontSize: "16px", opacity: 0.8 }}>
                  {selectedImage.description}
                </p>
              )}
              {selectedImage.tags && selectedImage.tags.length > 0 && (
                <div style={{ marginTop: "15px" }}>
                  {selectedImage.tags.map((tag) => (
                    <span
                      key={tag}
                      style={{
                        display: "inline-block",
                        padding: "5px 12px",
                        margin: "5px",
                        background: "rgba(255, 255, 255, 0.2)",
                        borderRadius: "15px",
                        fontSize: "14px",
                      }}
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
            <button
              onClick={() => setSelectedImage(null)}
              style={{
                marginTop: "20px",
                padding: "10px 30px",
                background: "rgba(255, 255, 255, 0.2)",
                border: "2px solid rgba(255, 255, 255, 0.5)",
                borderRadius: "25px",
                color: "white",
                fontSize: "16px",
                cursor: "pointer",
                transition: "all 0.3s",
              }}
              onMouseOver={(e) => {
                e.target.style.background = "rgba(255, 255, 255, 0.3)";
              }}
              onMouseOut={(e) => {
                e.target.style.background = "rgba(255, 255, 255, 0.2)";
              }}
              onMouseDown={(e) => e.stopPropagation()}
            >
              Close (ESC)
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

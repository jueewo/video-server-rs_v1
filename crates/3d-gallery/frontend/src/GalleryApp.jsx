import { useEffect, useRef, useState } from "preact/hooks";
import * as BABYLON from "@babylonjs/core";
import { fetchGalleryData } from "./api/galleryApi";
import {
  createGalleryRoom,
  getWallPositions,
  disposeGalleryRoom,
} from "./scene/GalleryRoom";
import {
  createImageFrames,
  setupFrameInteractions,
  disposeFrames,
} from "./scene/ImageFrame";
import {
  createVideoScreens,
  setupScreenInteractions,
  disposeVideoScreens,
  pauseAllVideos,
} from "./scene/VideoScreen";
import { VideoPlayer } from "./components/VideoPlayer";
import { Minimap } from "./components/Minimap";

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

    // Create camera - positioned inside the room
    const camera = new BABYLON.ArcRotateCamera(
      "camera",
      -Math.PI / 2, // Alpha (horizontal rotation) - facing north wall
      Math.PI / 2.5, // Beta (vertical rotation) - slight downward angle
      8, // Radius (distance from target) - closer to center
      new BABYLON.Vector3(0, 1.8, 0), // Target (look at eye level)
      scene,
    );
    camera.attachControl(canvasRef.current, true);
    camera.lowerRadiusLimit = 3;
    camera.upperRadiusLimit = 12;
    camera.wheelPrecision = 50;
    camera.lowerBetaLimit = 0.1;
    camera.upperBetaLimit = Math.PI / 2 - 0.1;

    // Don't prevent default on pick - allows clicking through to meshes
    camera.inputs.attached.pointers.buttons = [0, 1, 2]; // Left, middle, right

    // Store camera ref for later control detachment
    cameraRef.current = camera;

    // Fix aspect ratio
    camera.mode = BABYLON.Camera.PERSPECTIVE_CAMERA;
    const updateAspectRatio = () => {
      const canvas = canvasRef.current;
      if (canvas) {
        camera.getEngine().resize();
      }
    };
    updateAspectRatio();

    // Create the gallery room
    const room = createGalleryRoom(scene, {
      width: 16,
      depth: 16,
      height: 3.5,
    });

    // Store ceiling reference for dynamic transparency
    const ceiling = room.ceiling;
    const ceilingHeight = 3.5;

    // Get positions on walls for placing images
    const wallPositions = getWallPositions(room.walls, {
      itemsPerWall: 3,
      verticalOffset: 1.8,
      spacing: 4,
    });

    console.log(`Available wall positions: ${wallPositions.length}`);

    // Separate images and videos
    let frames = [];
    let videoScreens = [];

    if (galleryData.items && galleryData.items.length > 0) {
      console.log(`Processing ${galleryData.items.length} media items`);

      // Separate images and videos
      const images = galleryData.items
        .filter((item) => item.media_type === "image")
        .map((item) => ({
          id: item.id,
          title: item.title || "Untitled",
          description: item.description || "",
          url: item.url || item.thumbnail_url || "/placeholder.jpg",
          thumbnail_url: item.thumbnail_url || item.url,
          width: item.width,
          height: item.height,
          tags: item.tags || [],
          type: "image",
        }));

      const videos = galleryData.items
        .filter((item) => item.media_type === "video")
        .map((item) => ({
          id: item.id,
          title: item.title || "Untitled Video",
          description: item.description || "",
          url: item.url,
          thumbnail_url: item.thumbnail_url || item.url,
          width: item.width,
          height: item.height,
          tags: item.tags || [],
          duration: item.duration,
          type: "video",
        }));

      console.log(`Found ${images.length} images and ${videos.length} videos`);

      // Create image frames for images
      const imagePositions = wallPositions.slice(0, images.length);
      frames = createImageFrames(scene, images, imagePositions, {
        frameWidth: 2.5,
        frameThickness: 0.12,
      });

      // Create video screens for videos
      const videoPositions = wallPositions.slice(
        images.length,
        images.length + videos.length,
      );
      videoScreens = createVideoScreens(scene, videos, videoPositions, {
        screenWidth: 3.2,
        frameThickness: 0.15,
      });

      // Setup click interactions for images
      setupFrameInteractions(frames, (imageMetadata) => {
        console.log("Image clicked:", imageMetadata);
        setSelectedImage(imageMetadata);
      });

      // Setup click interactions for videos
      setupScreenInteractions(videoScreens, (videoMetadata) => {
        console.log("Video clicked:", videoMetadata);
        setSelectedImage(videoMetadata);
      });

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
    } else {
      console.warn("No media items in gallery data");
    }

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
        videoScreens.forEach((screen) => {
          if (screen.screenPlane) {
            screen.screenPlane.isVisible = screen.screenPlane.isInFrustum(
              scene.frustumPlanes,
            );
          }
          // Hide overlays too
          if (screen.playButtonOverlay) {
            screen.playButtonOverlay.isVisible =
              screen.screenPlane.isVisible && screen.videoElement.paused;
          }
          if (screen.progressBar && screen.progressBar.plane) {
            screen.progressBar.plane.isVisible =
              screen.screenPlane.isVisible && !screen.videoElement.paused;
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
      `Gallery contains ${frames.length} images and ${videoScreens.length} videos`,
    );

    // Cleanup function
    return () => {
      console.log("Cleaning up Babylon.js scene...");
      window.removeEventListener("resize", handleResize);
      engine.stopRenderLoop();

      // Pause all videos first
      if (videoScreens.length > 0) {
        pauseAllVideos(videoScreens);
      }

      // Dispose frames
      if (frames.length > 0) {
        disposeFrames(frames);
      }

      // Dispose video screens
      if (videoScreens.length > 0) {
        disposeVideoScreens(videoScreens);
      }

      // Dispose room
      disposeGalleryRoom(room);

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
      // ESC key to close overlay
      if (event.key === "Escape" && selectedImage) {
        setSelectedImage(null);
        return;
      }

      // Don't handle movement when overlay is open
      if (selectedImage) return;

      keysPressed[event.key.toLowerCase()] = true;
    };

    const handleKeyUp = (event) => {
      keysPressed[event.key.toLowerCase()] = false;
    };

    // Movement loop
    const moveInterval = setInterval(() => {
      if (selectedImage) return; // Don't move when overlay is open

      let moved = false;

      if (keysPressed["w"]) {
        camera.position.addInPlace(
          camera.getDirection(BABYLON.Vector3.Forward()).scale(moveSpeed),
        );
        moved = true;
      }
      if (keysPressed["s"]) {
        camera.position.addInPlace(
          camera.getDirection(BABYLON.Vector3.Forward()).scale(-moveSpeed),
        );
        moved = true;
      }
      if (keysPressed["a"]) {
        camera.position.addInPlace(
          camera.getDirection(BABYLON.Vector3.Left()).scale(moveSpeed),
        );
        moved = true;
      }
      if (keysPressed["d"]) {
        camera.position.addInPlace(
          camera.getDirection(BABYLON.Vector3.Right()).scale(moveSpeed),
        );
        moved = true;
      }

      // Clamp camera position to stay inside gallery
      if (moved) {
        camera.position.x = Math.max(-9, Math.min(9, camera.position.x));
        camera.position.z = Math.max(-9, Math.min(9, camera.position.z));
        camera.position.y = Math.max(0.5, Math.min(6, camera.position.y));
      }
    }, 16); // ~60fps

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      clearInterval(moveInterval);
    };
  }, [selectedImage]);

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
      {!loading && cameraRef.current && !selectedImage && (
        <Minimap camera={cameraRef.current} roomWidth={20} roomDepth={20} />
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

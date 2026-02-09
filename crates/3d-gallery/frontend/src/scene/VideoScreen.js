/**
 * VideoScreen.js
 *
 * Creates 3D video screens with playback controls for the gallery.
 * Handles video textures, play/pause, volume, and interactions.
 */

import * as BABYLON from "@babylonjs/core";
import Hls from "hls.js";

/**
 * Create a play button overlay plane for video screens
 */
function createPlayButtonOverlay(scene, width, height, videoId) {
  const overlayPlane = BABYLON.MeshBuilder.CreatePlane(
    `playOverlay_${videoId}`,
    { width: width * 0.5, height: height * 0.5 },
    scene,
  );

  // Simple bright red material for testing
  const overlayMaterial = new BABYLON.StandardMaterial(
    `playButtonMat_${videoId}`,
    scene,
  );
  overlayMaterial.diffuseColor = new BABYLON.Color3(1, 0, 0); // Bright red
  overlayMaterial.emissiveColor = new BABYLON.Color3(1, 0, 0); // Self-illuminated red
  overlayMaterial.backFaceCulling = false;
  overlayMaterial.needDepthPrePass = true;

  overlayPlane.material = overlayMaterial;
  overlayPlane.renderingGroupId = 3; // Render after everything else
  overlayPlane.alphaIndex = 1000; // Force to render last within group
  overlayPlane.isPickable = false; // Don't block clicks to video

  console.log("Play button overlay created (TEST RED):", {
    name: overlayPlane.name,
    width: width * 0.5,
    height: height * 0.5,
    renderingGroupId: overlayPlane.renderingGroupId,
    material: overlayMaterial.name,
  });

  return overlayPlane;
}

/**
 * Create a progress bar overlay for video playback
 */
function createProgressBarOverlay(scene, width, videoId) {
  const barHeight = 0.05;
  const barPlane = BABYLON.MeshBuilder.CreatePlane(
    `progressBar_${videoId}`,
    { width: width, height: barHeight },
    scene,
  );

  // Create canvas for progress bar
  const canvas = document.createElement("canvas");
  canvas.width = 1024;
  canvas.height = 32;

  const progressTexture = new BABYLON.DynamicTexture(
    `progressTexture_${videoId}`,
    canvas,
    scene,
    true,
  );
  progressTexture.hasAlpha = true;

  const progressMaterial = new BABYLON.StandardMaterial(
    `progressMat_${videoId}`,
    scene,
  );
  progressMaterial.diffuseTexture = progressTexture;
  progressMaterial.emissiveTexture = progressTexture;
  progressMaterial.opacityTexture = progressTexture;
  progressMaterial.backFaceCulling = false;
  progressMaterial.useAlphaFromDiffuseTexture = true;
  progressMaterial.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;

  barPlane.material = progressMaterial;
  barPlane.renderingGroupId = 2; // Render on top of video
  barPlane.isPickable = false;

  return { plane: barPlane, texture: progressTexture, canvas };
}

/**
 * Update progress bar based on video progress
 */
function updateProgressBar(progressBar, currentTime, duration) {
  if (!duration || duration === 0) return;

  const ctx = progressBar.canvas.getContext("2d");
  const progress = currentTime / duration;

  // Clear canvas
  ctx.clearRect(0, 0, progressBar.canvas.width, progressBar.canvas.height);

  // Draw background bar
  ctx.fillStyle = "rgba(255, 255, 255, 0.3)";
  ctx.fillRect(0, 0, progressBar.canvas.width, progressBar.canvas.height);

  // Draw progress bar
  ctx.fillStyle = "rgba(59, 130, 246, 0.9)"; // Blue
  ctx.fillRect(
    0,
    0,
    progressBar.canvas.width * progress,
    progressBar.canvas.height,
  );

  // Update texture
  progressBar.texture.update();
}

/**
 * Create a video screen with video texture and controls
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Object} videoData - Video data from API
 * @param {Object} options - Screen configuration
 * @returns {Object} Screen object with mesh, video element, and controls
 */
export function createVideoScreen(scene, videoData, options = {}) {
  const {
    position = new BABYLON.Vector3(0, 2, -5),
    rotation = new BABYLON.Vector3(0, 0, 0),
    width = 3.2,
    aspectRatio = 16 / 9,
    frameThickness = 0.15,
    frameColor = new BABYLON.Color3(0.05, 0.05, 0.05), // Black frame
    autoPlay = false,
  } = options;

  const height = width / aspectRatio;

  // Create parent node for the entire screen
  const screenParent = new BABYLON.TransformNode(
    `screen_${videoData.id}`,
    scene,
  );
  screenParent.position = position;
  screenParent.rotation = rotation;

  // Create HTML5 video element
  const videoElement = document.createElement("video");
  videoElement.crossOrigin = "anonymous";
  videoElement.loop = true;
  videoElement.muted = false;
  videoElement.volume = 0.5;
  videoElement.playsInline = true;
  videoElement.controls = false;

  if (autoPlay) {
    videoElement.autoplay = true;
    videoElement.muted = true; // Browsers require muted for autoplay
  }

  // Setup HLS.js for .m3u8 streams - but only if autoPlay is true
  // Otherwise, lazy load on first play
  let hls = null;
  let isInitialized = false;

  const initializeVideo = () => {
    if (isInitialized) return;
    isInitialized = true;

    if (videoData.url.includes(".m3u8")) {
      if (Hls.isSupported()) {
        hls = new Hls({
          debug: false,
          enableWorker: true,
          lowLatencyMode: false,
          backBufferLength: 90,
        });
        hls.loadSource(videoData.url);
        hls.attachMedia(videoElement);

        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          console.log(`✓ HLS manifest loaded for: ${videoData.title}`);
          if (autoPlay) {
            videoElement
              .play()
              .catch((err) => console.warn("Autoplay prevented:", err));
          }
        });

        hls.on(Hls.Events.ERROR, (event, data) => {
          console.error("HLS error:", data.type, data.details);
          if (data.fatal) {
            switch (data.type) {
              case Hls.ErrorTypes.NETWORK_ERROR:
                console.error("Fatal network error, trying to recover...");
                hls.startLoad();
                break;
              case Hls.ErrorTypes.MEDIA_ERROR:
                console.error("Fatal media error, trying to recover...");
                hls.recoverMediaError();
                break;
              default:
                console.error(
                  "Cannot recover from fatal error, destroying HLS instance",
                );
                hls.destroy();
                break;
            }
          }
        });
      } else if (videoElement.canPlayType("application/vnd.apple.mpegurl")) {
        // Safari native HLS support
        videoElement.src = videoData.url;
        console.log(`✓ Using native HLS for: ${videoData.title}`);
      } else {
        console.error("HLS not supported in this browser");
      }
    } else {
      // Direct video file (mp4, webm, etc.)
      videoElement.src = videoData.url;
    }
  };

  // Only initialize immediately if autoPlay is true
  if (autoPlay) {
    initializeVideo();
  }

  // Add error event listeners for debugging
  videoElement.addEventListener("error", (e) => {
    console.error(`❌ Video element error for ${videoData.title}:`, e);
    console.error("Video error details:", {
      error: videoElement.error,
      code: videoElement.error?.code,
      message: videoElement.error?.message,
      url: videoData.url,
    });
  });

  videoElement.addEventListener("loadedmetadata", () => {
    console.log(`✓ Video metadata loaded for ${videoData.title}:`, {
      duration: videoElement.duration,
      width: videoElement.videoWidth,
      height: videoElement.videoHeight,
    });
  });

  videoElement.addEventListener("canplay", () => {
    console.log(`✓ Video can play: ${videoData.title}`);
  });

  console.log(`Created video element for: ${videoData.title}`);

  // Create the video plane
  const screenPlane = BABYLON.MeshBuilder.CreatePlane(
    `videoPlane_${videoData.id}`,
    { width, height, sideOrientation: BABYLON.Mesh.DOUBLESIDE },
    scene,
  );
  screenPlane.parent = screenParent;
  screenPlane.position.z = 0.01; // Slight offset from frame

  // Create poster texture (shown before video loads)
  const posterTexture = new BABYLON.Texture(
    videoData.thumbnail_url || "/storage/images/video_placeholder.webp",
    scene,
  );

  // Create video texture
  const videoTexture = new BABYLON.VideoTexture(
    `videoTexture_${videoData.id}`,
    videoElement,
    scene,
    false, // generateMipMaps
    false, // invertY
    BABYLON.Texture.TRILINEAR_SAMPLINGMODE,
  );

  // Video orientation - flip horizontally for 180° rotated walls (South wall)
  // Check if the screen is rotated 180 degrees (π radians) on Y axis
  const isRotated180 =
    Math.abs(rotation.y - Math.PI) < 0.1 ||
    Math.abs(rotation.y + Math.PI) < 0.1;

  videoTexture.vScale = 1; // Keep normal vertical orientation
  videoTexture.uScale = isRotated180 ? -1 : 1; // Flip horizontally only for 180° walls

  // Apply same orientation to poster texture
  posterTexture.vScale = 1;
  posterTexture.uScale = isRotated180 ? -1 : 1;

  console.log(
    `Video ${videoData.title} - Rotation Y: ${rotation.y}, Flipped: ${isRotated180}`,
  );

  // Create screen material - start with poster
  const screenMaterial = new BABYLON.StandardMaterial(
    `screenMat_${videoData.id}`,
    scene,
  );
  screenMaterial.diffuseTexture = posterTexture;
  screenMaterial.emissiveTexture = posterTexture;
  screenMaterial.emissiveColor = new BABYLON.Color3(0.8, 0.8, 0.8);
  screenMaterial.specularColor = new BABYLON.Color3(0.05, 0.05, 0.05);
  screenMaterial.backFaceCulling = false;

  // Switch to video texture when video starts playing
  videoElement.addEventListener("playing", () => {
    screenMaterial.diffuseTexture = videoTexture;
    screenMaterial.emissiveTexture = videoTexture;
    console.log(`Switched to video texture for: ${videoData.title}`);
  });

  screenPlane.material = screenMaterial;
  screenPlane.renderingGroupId = 1;

  // Create play button overlay
  const playButtonOverlay = createPlayButtonOverlay(
    scene,
    width,
    height,
    videoData.id,
  );
  console.log(
    "Created play button overlay:",
    playButtonOverlay.name,
    "isVisible:",
    playButtonOverlay.isVisible,
  );
  // Parent back to screenParent for correct positioning
  playButtonOverlay.parent = screenParent;
  playButtonOverlay.position.x = 0;
  playButtonOverlay.position.y = 0;
  playButtonOverlay.position.z = 0.1; // In front of video plane
  playButtonOverlay.isVisible = true; // Show initially

  // Force the overlay to be refreshed on the next render
  scene.onAfterRenderObservable.addOnce(() => {
    playButtonOverlay.isVisible = true;
    console.log("Forced overlay visible on first render");
  });

  // Add callback to force overlay visible after poster loads
  posterTexture.onLoadObservable.add(() => {
    console.log(
      `Poster texture loaded for: ${videoData.title}, forcing overlay visible`,
    );
    playButtonOverlay.isVisible = true;
  });

  console.log("Play button overlay after setup:", {
    name: playButtonOverlay.name,
    position: playButtonOverlay.position,
    isVisible: playButtonOverlay.isVisible,
    renderingGroupId: playButtonOverlay.renderingGroupId,
    parent: playButtonOverlay.parent?.name,
  });

  // Create progress bar
  const progressBar = createProgressBarOverlay(scene, width, videoData.id);
  progressBar.plane.parent = screenParent;
  progressBar.plane.position.y = -height / 2 - 0.1; // Below screen
  progressBar.plane.position.z = 0.05;
  progressBar.plane.isVisible = false; // Hidden until video plays

  // Update progress bar periodically
  let progressInterval = null;
  videoElement.addEventListener("playing", () => {
    playButtonOverlay.isVisible = false;
    progressBar.plane.isVisible = true;

    if (!progressInterval) {
      progressInterval = setInterval(() => {
        if (!videoElement.paused) {
          updateProgressBar(
            progressBar,
            videoElement.currentTime,
            videoElement.duration,
          );
        }
      }, 100);
    }
  });

  videoElement.addEventListener("pause", () => {
    playButtonOverlay.isVisible = true;
  });

  videoElement.addEventListener("ended", () => {
    playButtonOverlay.isVisible = true;
    if (progressInterval) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  });

  // Create screen frame (like TV bezel)
  const frameBorder = createScreenFrame(
    scene,
    width,
    height,
    frameThickness,
    frameColor,
    videoData.id,
  );
  frameBorder.forEach((piece) => {
    piece.parent = screenParent;
    piece.isPickable = false;
  });

  // Add metadata for interactions
  screenPlane.metadata = {
    type: "video",
    id: videoData.id,
    title: videoData.title,
    description: videoData.description,
    url: videoData.url,
    thumbnail_url: videoData.thumbnail_url,
    duration: videoData.duration,
    tags: videoData.tags || [],
    videoElement: videoElement,
    isPlaying: autoPlay,
    playButtonOverlay: playButtonOverlay,
    progressBar: progressBar,
    initializeVideo: initializeVideo, // Store for lazy loading
  };

  // Make the screen clickable
  screenPlane.isPickable = true;

  // Add hover and click interactions
  setupVideoInteractions(
    screenPlane,
    videoElement,
    screenMaterial,
    playButtonOverlay,
  );

  console.log(`Created video screen for: ${videoData.title || videoData.id}`);

  return {
    parent: screenParent,
    screenPlane,
    videoElement,
    videoTexture,
    posterTexture,
    frameBorder,
    playButtonOverlay,
    progressBar,
    metadata: screenPlane.metadata,
    hls, // Store HLS instance for cleanup
    initializeVideo, // Store initialization function for lazy loading
  };
}

/**
 * Create the screen frame/bezel
 */
function createScreenFrame(scene, width, height, thickness, color, id) {
  const border = [];
  const depth = thickness / 2;

  const frameMaterial = new BABYLON.StandardMaterial(`frameMat_${id}`, scene);
  frameMaterial.diffuseColor = color;
  frameMaterial.specularColor = new BABYLON.Color3(0.2, 0.2, 0.2);
  frameMaterial.emissiveColor = new BABYLON.Color3(0.02, 0.02, 0.02); // Slight glow

  // Top border
  const top = BABYLON.MeshBuilder.CreateBox(
    `frameTop_${id}`,
    { width: width + thickness * 2, height: thickness, depth },
    scene,
  );
  top.position.y = height / 2 + thickness / 2;
  top.position.z = -depth / 2;
  top.material = frameMaterial;
  top.isPickable = false;
  border.push(top);

  // Bottom border
  const bottom = BABYLON.MeshBuilder.CreateBox(
    `frameBottom_${id}`,
    { width: width + thickness * 2, height: thickness, depth },
    scene,
  );
  bottom.position.y = -height / 2 - thickness / 2;
  bottom.position.z = -depth / 2;
  bottom.material = frameMaterial;
  bottom.isPickable = false;
  border.push(bottom);

  // Left border
  const left = BABYLON.MeshBuilder.CreateBox(
    `frameLeft_${id}`,
    { width: thickness, height: height, depth },
    scene,
  );
  left.position.x = -width / 2 - thickness / 2;
  left.position.z = -depth / 2;
  left.material = frameMaterial;
  left.isPickable = false;
  border.push(left);

  // Right border
  const right = BABYLON.MeshBuilder.CreateBox(
    `frameRight_${id}`,
    { width: thickness, height: height, depth },
    scene,
  );
  right.position.x = width / 2 + thickness / 2;
  right.position.z = -depth / 2;
  right.material = frameMaterial;
  right.isPickable = false;
  border.push(right);

  return border;
}

/**
 * Setup video playback interactions (play/pause on click, hover effects)
 */
function setupVideoInteractions(
  screenPlane,
  videoElement,
  material,
  playButtonOverlay,
) {
  if (!screenPlane.actionManager) {
    screenPlane.actionManager = new BABYLON.ActionManager(
      screenPlane.getScene(),
    );
  }

  let hoverTimeout = null;

  // Hover effect - increase emissive and start video preview
  screenPlane.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPointerOverTrigger,
      () => {
        material.emissiveColor = new BABYLON.Color3(1.0, 1.0, 1.0);

        // Start video after 500ms hover
        hoverTimeout = setTimeout(() => {
          if (videoElement.paused && screenPlane.metadata) {
            // Initialize if needed
            const metadata = screenPlane.metadata;
            const screen = {
              videoElement: videoElement,
              initializeVideo: metadata.initializeVideo || (() => {}),
              metadata: metadata,
            };
            playVideo(screen);
          }
        }, 500);
      },
    ),
  );

  screenPlane.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPointerOutTrigger,
      () => {
        material.emissiveColor = new BABYLON.Color3(0.8, 0.8, 0.8);

        // Cancel hover autoplay
        if (hoverTimeout) {
          clearTimeout(hoverTimeout);
          hoverTimeout = null;
        }
      },
    ),
  );

  // Click to toggle play/pause (handled separately via observable)
}

/**
 * Create multiple video screens and position them on walls
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Array} videos - Array of video data objects
 * @param {Array} positions - Array of position objects
 * @param {Object} options - Screen options
 * @returns {Array} Array of screen objects
 */
export function createVideoScreens(scene, videos, positions, options = {}) {
  const screens = [];

  videos.forEach((videoData, index) => {
    if (index >= positions.length) {
      console.warn(`No position available for video ${index}`);
      return;
    }

    const pos = positions[index];

    // Adjust screen size based on video aspect ratio if available
    let aspectRatio = 16 / 9; // Default
    if (videoData.width && videoData.height) {
      aspectRatio = videoData.width / videoData.height;
    }

    const screen = createVideoScreen(scene, videoData, {
      position: pos.position,
      rotation: pos.rotation,
      width: options.screenWidth || 3.2,
      aspectRatio,
      frameThickness: options.frameThickness || 0.15,
      frameColor: options.frameColor,
      autoPlay: false, // Don't autoplay videos in room - only when clicked
    });

    screens.push(screen);
  });

  console.log(`Created ${screens.length} video screens`);
  return screens;
}

/**
 * Setup click interactions for all video screens
 *
 * @param {Array} screens - Array of screen objects
 * @param {Function} onVideoClick - Callback when video is clicked (for overlay)
 */
export function setupScreenInteractions(screens, onVideoClick) {
  screens.forEach((screen) => {
    const screenPlane = screen.screenPlane;
    const videoElement = screen.videoElement;

    if (!screenPlane.actionManager) {
      screenPlane.actionManager = new BABYLON.ActionManager(
        screenPlane.getScene(),
      );
    }

    // Add click action for play/pause toggle
    screenPlane.actionManager.registerAction(
      new BABYLON.ExecuteCodeAction(
        BABYLON.ActionManager.OnPickDownTrigger,
        () => {
          toggleVideoPlayback(
            videoElement,
            screen.metadata,
            screen.initializeVideo,
          );

          // Also call the overlay callback if provided
          if (onVideoClick) {
            console.log("Video clicked:", screen.metadata.title);
            onVideoClick(screen.metadata);
          }
        },
      ),
    );
  });

  console.log(`Setup interactions for ${screens.length} video screens`);
}

/**
 * Toggle video playback (play/pause)
 */
function toggleVideoPlayback(videoElement, metadata, initializeVideo) {
  if (videoElement.paused) {
    // Initialize video on first play (lazy loading)
    if (initializeVideo) {
      initializeVideo();
    }
    videoElement
      .play()
      .then(() => {
        metadata.isPlaying = true;
        console.log(`▶ Playing: ${metadata.title}`);
      })
      .catch((err) => {
        console.error("Failed to play video:", err);
      });
  } else {
    videoElement.pause();
    metadata.isPlaying = false;
    console.log(`⏸ Paused: ${metadata.title}`);
  }
}

/**
 * Play a video
 */
export function playVideo(screen) {
  if (screen.videoElement && screen.videoElement.paused) {
    // Initialize video on first play (lazy loading)
    if (screen.initializeVideo) {
      screen.initializeVideo();
    }
    screen.videoElement
      .play()
      .then(() => {
        if (screen.metadata) {
          screen.metadata.isPlaying = true;
          console.log(`▶ Playing: ${screen.metadata.title}`);
        }
      })
      .catch((err) => {
        console.error("Failed to play video:", err);
      });
  }
}

/**
 * Pause a video
 */
export function pauseVideo(screen) {
  if (screen.videoElement && !screen.videoElement.paused) {
    screen.videoElement.pause();
    screen.metadata.isPlaying = false;
    console.log(`⏸ Paused: ${screen.metadata.title}`);
  }
}

/**
 * Set video volume (0-1)
 */
export function setVideoVolume(screen, volume) {
  if (screen.videoElement) {
    screen.videoElement.volume = Math.max(0, Math.min(1, volume));
    console.log(`🔊 Volume set to ${Math.round(volume * 100)}%`);
  }
}

/**
 * Mute/unmute video
 */
export function toggleVideoMute(screen) {
  if (screen.videoElement) {
    screen.videoElement.muted = !screen.videoElement.muted;
    console.log(screen.videoElement.muted ? "🔇 Muted" : "🔊 Unmuted");
  }
}

/**
 * Dispose of a video screen and all its components
 */
export function disposeVideoScreen(screen) {
  // Destroy HLS instance first
  if (screen.hls) {
    screen.hls.destroy();
    screen.hls = null;
  }

  // Pause and dispose video element
  if (screen.videoElement) {
    screen.videoElement.pause();
    screen.videoElement.src = "";
    screen.videoElement.load();
  }

  // Dispose video texture
  if (screen.videoTexture) {
    screen.videoTexture.dispose();
  }

  // Dispose poster texture
  if (screen.posterTexture) {
    screen.posterTexture.dispose();
  }

  // Dispose play button overlay
  if (screen.playButtonOverlay) {
    if (screen.playButtonOverlay.material) {
      screen.playButtonOverlay.material.dispose();
    }
    screen.playButtonOverlay.dispose();
  }

  // Dispose progress bar
  if (screen.progressBar) {
    if (screen.progressBar.plane) {
      if (screen.progressBar.plane.material) {
        screen.progressBar.plane.material.dispose();
      }
      screen.progressBar.plane.dispose();
    }
    if (screen.progressBar.texture) {
      screen.progressBar.texture.dispose();
    }
  }

  // Dispose screen plane
  if (screen.screenPlane) {
    if (screen.screenPlane.material) {
      screen.screenPlane.material.dispose();
    }
    screen.screenPlane.dispose();
  }

  // Dispose frame borders
  screen.frameBorder?.forEach((piece) => piece.dispose());

  // Dispose parent
  screen.parent?.dispose();
}

/**
 * Dispose of all video screens
 */
export function disposeVideoScreens(screens) {
  screens.forEach((screen) => disposeVideoScreen(screen));
  console.log(`Disposed ${screens.length} video screens`);
}

/**
 * Pause all videos
 */
export function pauseAllVideos(screens) {
  screens.forEach((screen) => pauseVideo(screen));
  console.log("All videos paused");
}

/**
 * Get video playback state
 */
export function getVideoState(screen) {
  if (!screen.videoElement) return null;

  return {
    isPlaying: !screen.videoElement.paused,
    currentTime: screen.videoElement.currentTime,
    duration: screen.videoElement.duration,
    volume: screen.videoElement.volume,
    muted: screen.videoElement.muted,
    progress: screen.videoElement.currentTime / screen.videoElement.duration,
  };
}

/**
 * VideoScreen.js
 *
 * Creates 3D video screens with playback controls for the gallery.
 * Handles video textures, play/pause, volume, and interactions.
 */

import * as BABYLON from "@babylonjs/core";

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
  videoElement.src = videoData.url;
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

  console.log(`Created video element for: ${videoData.title}`);

  // Create the video plane
  const screenPlane = BABYLON.MeshBuilder.CreatePlane(
    `videoPlane_${videoData.id}`,
    { width, height, sideOrientation: BABYLON.Mesh.DOUBLESIDE },
    scene,
  );
  screenPlane.parent = screenParent;
  screenPlane.position.z = 0.01; // Slight offset from frame

  // Create video texture
  const videoTexture = new BABYLON.VideoTexture(
    `videoTexture_${videoData.id}`,
    videoElement,
    scene,
    false, // generateMipMaps
    false, // invertY
    BABYLON.Texture.TRILINEAR_SAMPLINGMODE,
  );

  // Fix orientation to match images
  videoTexture.vScale = 1; // Keep normal vertical orientation
  videoTexture.uScale = -1; // Flip horizontal to match images

  // Create screen material
  const screenMaterial = new BABYLON.StandardMaterial(
    `screenMat_${videoData.id}`,
    scene,
  );
  screenMaterial.diffuseTexture = videoTexture;
  screenMaterial.emissiveTexture = videoTexture;
  screenMaterial.emissiveColor = new BABYLON.Color3(0.8, 0.8, 0.8);
  screenMaterial.specularColor = new BABYLON.Color3(0.05, 0.05, 0.05);
  screenMaterial.backFaceCulling = false;

  screenPlane.material = screenMaterial;
  screenPlane.renderingGroupId = 1;

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
  };

  // Make the screen clickable
  screenPlane.isPickable = true;

  // Add hover and click interactions
  setupVideoInteractions(screenPlane, videoElement, screenMaterial);

  console.log(`Created video screen for: ${videoData.title || videoData.id}`);

  return {
    parent: screenParent,
    screenPlane,
    videoElement,
    videoTexture,
    frameBorder,
    metadata: screenPlane.metadata,
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
function setupVideoInteractions(screenPlane, videoElement, material) {
  if (!screenPlane.actionManager) {
    screenPlane.actionManager = new BABYLON.ActionManager(
      screenPlane.getScene(),
    );
  }

  // Hover effect - increase emissive slightly
  screenPlane.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPointerOverTrigger,
      () => {
        material.emissiveColor = new BABYLON.Color3(1.0, 1.0, 1.0);
      },
    ),
  );

  screenPlane.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPointerOutTrigger,
      () => {
        material.emissiveColor = new BABYLON.Color3(0.8, 0.8, 0.8);
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
      autoPlay: false,
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
          toggleVideoPlayback(videoElement, screen.metadata);

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
function toggleVideoPlayback(videoElement, metadata) {
  if (videoElement.paused) {
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
    screen.videoElement
      .play()
      .then(() => {
        screen.metadata.isPlaying = true;
        console.log(`▶ Playing: ${screen.metadata.title}`);
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

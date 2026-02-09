/**
 * ImageFrame.js
 *
 * Creates 3D picture frames with image textures for the gallery.
 * Handles image loading, frame creation, and interactions.
 */

import * as BABYLON from "@babylonjs/core";

/**
 * Create a picture frame with an image texture
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Object} imageData - Image data from API
 * @param {Object} options - Frame configuration
 * @returns {Object} Frame object with mesh and metadata
 */
export function createImageFrame(scene, imageData, options = {}) {
  const {
    position = new BABYLON.Vector3(0, 2, -5),
    rotation = new BABYLON.Vector3(0, 0, 0),
    width = 2,
    aspectRatio = 16 / 9,
    frameThickness = 0.1,
    frameColor = new BABYLON.Color3(0.1, 0.08, 0.05), // Dark wood
  } = options;

  const height = width / aspectRatio;

  // Create parent node for the entire frame
  const frameParent = new BABYLON.TransformNode(`frame_${imageData.id}`, scene);
  frameParent.position = position;
  frameParent.rotation = rotation;

  // Create the image plane (double-sided to ensure visibility)
  const imagePlane = BABYLON.MeshBuilder.CreatePlane(
    `image_${imageData.id}`,
    { width, height, sideOrientation: BABYLON.Mesh.DOUBLESIDE },
    scene,
  );
  imagePlane.parent = frameParent;
  imagePlane.position.z = 0.01; // Slight offset from frame

  // Create image material with texture
  const imageMaterial = new BABYLON.StandardMaterial(
    `imageMat_${imageData.id}`,
    scene,
  );

  // Load the image texture with error handling
  console.log(`Loading texture for ${imageData.title} from: ${imageData.url}`);

  const texture = new BABYLON.Texture(
    imageData.url,
    scene,
    false,
    false,
    BABYLON.Texture.TRILINEAR_SAMPLINGMODE,
    () => {
      // onLoad callback
      console.log(`✓ Texture loaded: ${imageData.title}`, {
        width: texture.getSize().width,
        height: texture.getSize().height,
        isReady: texture.isReady(),
      });
    },
    (message, exception) => {
      // onError callback
      console.error(
        `✗ Failed to load texture: ${imageData.url}`,
        message,
        exception,
      );
      console.warn(`Using placeholder color for: ${imageData.title}`);
      // Use a solid color as fallback
      imageMaterial.diffuseTexture = null;
      imageMaterial.diffuseColor = new BABYLON.Color3(0.7, 0.7, 0.8);
      imageMaterial.emissiveColor = new BABYLON.Color3(0.2, 0.2, 0.25);
    },
  );

  texture.hasAlpha = false;
  texture.vScale = -1; // Flip vertically to correct upside-down images
  texture.uScale = -1; // Flip horizontally to correct mirroring

  imageMaterial.diffuseTexture = texture;
  imageMaterial.specularColor = new BABYLON.Color3(0.1, 0.1, 0.1);
  imageMaterial.emissiveColor = new BABYLON.Color3(0.15, 0.15, 0.15); // Slight glow for visibility
  imageMaterial.backFaceCulling = false; // Render both sides
  imageMaterial.alphaMode = BABYLON.Engine.ALPHA_DISABLE;

  imagePlane.material = imageMaterial;
  imagePlane.renderingGroupId = 1; // Render after walls

  console.log(`Frame plane created:`, {
    id: imageData.id,
    title: imageData.title,
    position: position,
    rotation: rotation,
    width,
    height,
    hasTexture: !!imageMaterial.diffuseTexture,
  });

  // Create frame border (4 pieces)
  const frameBorder = createFrameBorder(
    scene,
    width,
    height,
    frameThickness,
    frameColor,
    imageData.id,
  );
  frameBorder.forEach((piece) => {
    piece.parent = frameParent;
    piece.isPickable = false; // Don't block clicks to the image
  });

  // Add metadata for interactions
  imagePlane.metadata = {
    type: "image",
    id: imageData.id,
    title: imageData.title,
    description: imageData.description,
    url: imageData.url,
    thumbnail_url: imageData.thumbnail_url,
    tags: imageData.tags || [],
  };

  // Make the image clickable
  imagePlane.isPickable = true;

  // Add subtle hover glow effect
  imagePlane.actionManager = new BABYLON.ActionManager(scene);

  imagePlane.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPointerOverTrigger,
      () => {
        imageMaterial.emissiveColor = new BABYLON.Color3(0.2, 0.2, 0.2);
      },
    ),
  );

  imagePlane.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPointerOutTrigger,
      () => {
        imageMaterial.emissiveColor = new BABYLON.Color3(0.1, 0.1, 0.1);
      },
    ),
  );

  console.log(`Created frame for image: ${imageData.title || imageData.id}`);

  return {
    parent: frameParent,
    imagePlane,
    frameBorder,
    metadata: imagePlane.metadata,
  };
}

/**
 * Create the wooden frame border around the image
 */
function createFrameBorder(scene, width, height, thickness, color, id) {
  const border = [];
  const depth = thickness / 2;

  const frameMaterial = new BABYLON.StandardMaterial(`frameMat_${id}`, scene);
  frameMaterial.diffuseColor = color;
  frameMaterial.specularColor = new BABYLON.Color3(0.3, 0.25, 0.2);

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
 * Create multiple image frames and position them on walls
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Array} images - Array of image data objects
 * @param {Array} positions - Array of position objects from getWallPositions
 * @param {Object} options - Frame options
 * @returns {Array} Array of frame objects
 */
export function createImageFrames(scene, images, positions, options = {}) {
  const frames = [];

  images.forEach((imageData, index) => {
    if (index >= positions.length) {
      console.warn(`No position available for image ${index}`);
      return;
    }

    const pos = positions[index];

    // Adjust frame size based on image aspect ratio if available
    let aspectRatio = 16 / 9; // Default
    if (imageData.width && imageData.height) {
      aspectRatio = imageData.width / imageData.height;
    }

    const frame = createImageFrame(scene, imageData, {
      position: pos.position,
      rotation: pos.rotation,
      width: options.frameWidth || 2,
      aspectRatio,
      frameThickness: options.frameThickness || 0.1,
      frameColor: options.frameColor,
    });

    frames.push(frame);
  });

  console.log(`Created ${frames.length} image frames`);
  return frames;
}

/**
 * Setup click interactions for all frames
 *
 * @param {Array} frames - Array of frame objects
 * @param {Function} onImageClick - Callback when image is clicked
 */
export function setupFrameInteractions(frames, onImageClick) {
  frames.forEach((frame) => {
    const imagePlane = frame.imagePlane;

    if (!imagePlane.actionManager) {
      imagePlane.actionManager = new BABYLON.ActionManager(
        imagePlane.getScene(),
      );
    }

    // Add click action with high priority (executes before camera drag)
    imagePlane.actionManager.registerAction(
      new BABYLON.ExecuteCodeAction(
        BABYLON.ActionManager.OnPickDownTrigger,
        () => {
          if (onImageClick) {
            console.log("Image pick triggered:", frame.metadata.title);
            onImageClick(frame.metadata);
          }
        },
      ),
    );
  });

  console.log(`Setup interactions for ${frames.length} frames`);
}

/**
 * Dispose of a frame and all its components
 */
export function disposeFrame(frame) {
  if (frame.imagePlane) {
    if (frame.imagePlane.material) {
      if (frame.imagePlane.material.diffuseTexture) {
        frame.imagePlane.material.diffuseTexture.dispose();
      }
      frame.imagePlane.material.dispose();
    }
    frame.imagePlane.dispose();
  }

  frame.frameBorder?.forEach((piece) => piece.dispose());
  frame.parent?.dispose();
}

/**
 * Dispose of all frames
 */
export function disposeFrames(frames) {
  frames.forEach((frame) => disposeFrame(frame));
  console.log(`Disposed ${frames.length} frames`);
}

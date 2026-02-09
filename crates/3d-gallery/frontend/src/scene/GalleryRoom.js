/**
 * GalleryRoom.js
 *
 * Creates a 3D gallery room environment with walls, floor, ceiling, and lighting.
 * Designed for displaying images and videos in an immersive space.
 */

import * as BABYLON from "@babylonjs/core";

/**
 * Create a complete gallery room with walls, floor, ceiling, and lighting
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Object} options - Room configuration options
 * @returns {Object} Room components (walls, floor, ceiling, lights)
 */
export function createGalleryRoom(scene, options = {}) {
  const {
    width = 20,
    depth = 20,
    height = 4,
    wallColor = new BABYLON.Color3(0.95, 0.95, 0.95), // Light gray walls
    floorColor = new BABYLON.Color3(0.3, 0.25, 0.2), // Dark wood floor
    ceilingColor = new BABYLON.Color3(0.9, 0.9, 0.9), // Light ceiling
  } = options;

  const room = {
    walls: [],
    floor: null,
    ceiling: null,
    lights: [],
  };

  // Create floor
  room.floor = createFloor(scene, width, depth, floorColor);

  // Create ceiling
  room.ceiling = createCeiling(scene, width, depth, height, ceilingColor);

  // Create 4 walls
  room.walls = createWalls(scene, width, depth, height, wallColor);

  // Create gallery lighting
  room.lights = createGalleryLighting(scene, width, depth, height);

  console.log("Gallery room created:", {
    dimensions: { width, depth, height },
    walls: room.walls.length,
    lights: room.lights.length,
  });

  return room;
}

/**
 * Create the floor of the gallery
 */
function createFloor(scene, width, depth, color) {
  const floor = BABYLON.MeshBuilder.CreateGround(
    "floor",
    { width, height: depth },
    scene,
  );

  const floorMaterial = new BABYLON.StandardMaterial("floorMaterial", scene);
  floorMaterial.diffuseColor = color;
  floorMaterial.specularColor = new BABYLON.Color3(0.1, 0.1, 0.1);
  floorMaterial.backFaceCulling = false;

  // Add subtle texture pattern
  floorMaterial.bumpTexture = createProceduralTexture(
    scene,
    "floorBump",
    color,
  );

  floor.material = floorMaterial;
  floor.position.y = 0;
  floor.receiveShadows = true;

  return floor;
}

/**
 * Create the ceiling of the gallery
 */
function createCeiling(scene, width, depth, height, color) {
  const ceiling = BABYLON.MeshBuilder.CreatePlane(
    "ceiling",
    { width, height: depth },
    scene,
  );

  const ceilingMaterial = new BABYLON.StandardMaterial(
    "ceilingMaterial",
    scene,
  );
  ceilingMaterial.diffuseColor = color;
  ceilingMaterial.specularColor = new BABYLON.Color3(0.05, 0.05, 0.05);
  ceilingMaterial.backFaceCulling = false;

  ceiling.material = ceilingMaterial;
  ceiling.position.y = height;
  ceiling.rotation.x = Math.PI / 2; // Face downward

  return ceiling;
}

/**
 * Create 4 walls of the gallery room
 */
function createWalls(scene, width, depth, height, color) {
  const walls = [];
  const wallMaterial = new BABYLON.StandardMaterial("wallMaterial", scene);
  wallMaterial.diffuseColor = color;
  wallMaterial.specularColor = new BABYLON.Color3(0.1, 0.1, 0.1);
  wallMaterial.backFaceCulling = false; // Visible from both sides

  // Wall configurations: [position, rotation, width]
  const wallConfigs = [
    // North wall (back)
    {
      name: "northWall",
      position: new BABYLON.Vector3(0, height / 2, -depth / 2),
      rotation: new BABYLON.Vector3(0, 0, 0),
      width: width,
    },
    // South wall (front)
    {
      name: "southWall",
      position: new BABYLON.Vector3(0, height / 2, depth / 2),
      rotation: new BABYLON.Vector3(0, Math.PI, 0),
      width: width,
    },
    // East wall (right)
    {
      name: "eastWall",
      position: new BABYLON.Vector3(width / 2, height / 2, 0),
      rotation: new BABYLON.Vector3(0, Math.PI / 2, 0),
      width: depth,
    },
    // West wall (left)
    {
      name: "westWall",
      position: new BABYLON.Vector3(-width / 2, height / 2, 0),
      rotation: new BABYLON.Vector3(0, -Math.PI / 2, 0),
      width: depth,
    },
  ];

  wallConfigs.forEach((config) => {
    const wall = BABYLON.MeshBuilder.CreatePlane(
      config.name,
      { width: config.width, height: height },
      scene,
    );

    wall.position = config.position;
    wall.rotation = config.rotation;
    wall.material = wallMaterial;
    wall.receiveShadows = true;

    // Store wall metadata for later use (image placement)
    wall.metadata = {
      type: "wall",
      name: config.name,
      normal: config.rotation.y,
    };

    walls.push(wall);
  });

  return walls;
}

/**
 * Create gallery lighting setup
 */
function createGalleryLighting(scene, width, depth, height) {
  const lights = [];

  // Ambient light (soft overall illumination)
  const ambientLight = new BABYLON.HemisphericLight(
    "ambientLight",
    new BABYLON.Vector3(0, 1, 0),
    scene,
  );
  ambientLight.intensity = 0.5;
  ambientLight.diffuse = new BABYLON.Color3(1, 1, 0.98);
  lights.push(ambientLight);

  // Directional light (main light source from above)
  const mainLight = new BABYLON.DirectionalLight(
    "mainLight",
    new BABYLON.Vector3(0.5, -1, 0.3),
    scene,
  );
  mainLight.position = new BABYLON.Vector3(0, height * 0.8, 0);
  mainLight.intensity = 0.6;
  mainLight.diffuse = new BABYLON.Color3(1, 1, 1);
  lights.push(mainLight);

  // Spotlights for gallery feel (positioned at ceiling corners)
  const spotlightPositions = [
    new BABYLON.Vector3(width * 0.3, height * 0.9, depth * 0.3),
    new BABYLON.Vector3(-width * 0.3, height * 0.9, depth * 0.3),
    new BABYLON.Vector3(width * 0.3, height * 0.9, -depth * 0.3),
    new BABYLON.Vector3(-width * 0.3, height * 0.9, -depth * 0.3),
  ];

  spotlightPositions.forEach((pos, index) => {
    const spotlight = new BABYLON.SpotLight(
      `spotlight${index}`,
      pos,
      new BABYLON.Vector3(0, -1, 0),
      Math.PI / 3,
      2,
      scene,
    );
    spotlight.intensity = 0.4;
    spotlight.diffuse = new BABYLON.Color3(1, 0.98, 0.95);
    lights.push(spotlight);
  });

  console.log(`Created ${lights.length} lights for gallery`);
  return lights;
}

/**
 * Create a procedural texture for subtle surface detail
 */
function createProceduralTexture(scene, name, baseColor) {
  // For now, return null - can be enhanced later with actual procedural textures
  // This would require @babylonjs/procedural-textures package
  return null;
}

/**
 * Get wall positions for placing images/frames
 * Returns an array of available positions on each wall
 *
 * @param {Array} walls - Array of wall meshes
 * @param {Object} options - Configuration options
 * @returns {Array} Array of position objects
 */
export function getWallPositions(walls, options = {}) {
  const {
    itemsPerWall = 3,
    verticalOffset = 2, // Height from floor
    spacing = 3, // Space between items
  } = options;

  const positions = [];

  walls.forEach((wall) => {
    const wallWidth = wall.getBoundingInfo().boundingBox.extendSize.x * 2;
    const totalWidth = (itemsPerWall - 1) * spacing;
    const startX = -totalWidth / 2;
    const offset = 0.15; // Offset from wall to prevent z-fighting

    for (let i = 0; i < itemsPerWall; i++) {
      const localX = startX + i * spacing;

      // Convert local position to world position based on wall rotation
      // Place images SLIGHTLY IN FRONT of walls
      let worldPos;
      const wallName = wall.name;

      if (wallName === "northWall") {
        // North wall faces south (positive Z), place images in front (positive Z)
        worldPos = new BABYLON.Vector3(
          localX,
          verticalOffset,
          wall.position.z + offset,
        );
      } else if (wallName === "southWall") {
        // South wall faces north (negative Z), place images in front (negative Z)
        worldPos = new BABYLON.Vector3(
          -localX,
          verticalOffset,
          wall.position.z - offset,
        );
      } else if (wallName === "eastWall") {
        // East wall faces west (negative X), place images in front (negative X)
        worldPos = new BABYLON.Vector3(
          wall.position.x - offset,
          verticalOffset,
          -localX,
        );
      } else if (wallName === "westWall") {
        // West wall faces east (positive X), place images in front (positive X)
        worldPos = new BABYLON.Vector3(
          wall.position.x + offset,
          verticalOffset,
          localX,
        );
      }

      positions.push({
        position: worldPos,
        rotation: wall.rotation.clone(),
        wall: wallName,
        index: i,
      });
    }
  });

  return positions;
}

/**
 * Dispose of all room elements
 */
export function disposeGalleryRoom(room) {
  if (room.floor) room.floor.dispose();
  if (room.ceiling) room.ceiling.dispose();

  room.walls.forEach((wall) => wall.dispose());
  room.lights.forEach((light) => light.dispose());

  console.log("Gallery room disposed");
}

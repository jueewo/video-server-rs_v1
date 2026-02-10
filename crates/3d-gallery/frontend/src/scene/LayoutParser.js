/**
 * LayoutParser.js
 *
 * Parses JSON gallery layouts and generates 3D rooms with walls, floors, ceilings,
 * and media placement slots using Babylon.js.
 */

import * as BABYLON from "@babylonjs/core";

/**
 * Parse a gallery layout JSON and create all rooms in the scene
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Object} layout - The parsed JSON layout object
 * @returns {Object} Generated gallery structure
 */
export function createGalleryFromLayout(scene, layout) {
  console.log(`Creating gallery: ${layout.name}`);
  console.log(`Description: ${layout.description}`);

  const gallery = {
    name: layout.name,
    rooms: [],
    walls: [],
    slots: [],
    doorways: [],
    lights: [],
  };

  // Create each room
  layout.rooms.forEach((roomConfig) => {
    const room = createRoom(scene, roomConfig);
    gallery.rooms.push(room);
    gallery.walls.push(...room.walls);
    gallery.slots.push(...room.slots);
    gallery.doorways.push(...room.doorways);
    gallery.lights.push(...room.lights);
  });

  console.log(
    `Gallery created: ${gallery.rooms.length} rooms, ${gallery.walls.length} walls, ${gallery.slots.length} slots`,
  );

  return gallery;
}

/**
 * Create a single room from configuration
 *
 * @param {BABYLON.Scene} scene - The Babylon.js scene
 * @param {Object} roomConfig - Room configuration from JSON
 * @returns {Object} Room structure
 */
function createRoom(scene, roomConfig) {
  const { id, name, position, dimensions, floor_color, ceiling_color } =
    roomConfig;
  const [x, y, z] = position;

  console.log(`Creating room: ${name} at position [${x}, ${y}, ${z}]`);

  const room = {
    id,
    name,
    position: new BABYLON.Vector3(x, y, z),
    dimensions,
    floor: null,
    ceiling: null,
    walls: [],
    slots: [],
    doorways: [],
    lights: [],
  };

  // Create floor
  room.floor = createFloor(
    scene,
    roomConfig,
    new BABYLON.Vector3(x, y, z),
    floor_color,
  );

  // Create ceiling
  room.ceiling = createCeiling(
    scene,
    roomConfig,
    new BABYLON.Vector3(x, y + dimensions.height, z),
    ceiling_color,
  );

  // Create walls with slots
  roomConfig.walls.forEach((wallConfig) => {
    const wallData = createWall(scene, roomConfig, wallConfig);
    room.walls.push(wallData.wall);
    room.slots.push(...wallData.slots);
    if (wallData.doorway) {
      room.doorways.push(wallData.doorway);
    }
  });

  // Create lighting for the room
  room.lights = createRoomLighting(scene, roomConfig);

  return room;
}

/**
 * Create a floor mesh for a room
 */
function createFloor(scene, roomConfig, position, color) {
  const { width, depth } = roomConfig.dimensions;

  const floor = BABYLON.MeshBuilder.CreateGround(
    `floor_${roomConfig.id}`,
    { width, height: depth },
    scene,
  );

  const floorMaterial = new BABYLON.StandardMaterial(
    `floorMat_${roomConfig.id}`,
    scene,
  );
  floorMaterial.diffuseColor = new BABYLON.Color3(...color);
  floorMaterial.specularColor = new BABYLON.Color3(0.1, 0.1, 0.1);

  floor.material = floorMaterial;
  floor.position = new BABYLON.Vector3(position.x, position.y, position.z);
  floor.receiveShadows = true;

  return floor;
}

/**
 * Create a ceiling mesh for a room
 */
function createCeiling(scene, roomConfig, position, color) {
  const { width, depth } = roomConfig.dimensions;

  const ceiling = BABYLON.MeshBuilder.CreatePlane(
    `ceiling_${roomConfig.id}`,
    { width, height: depth },
    scene,
  );

  const ceilingMaterial = new BABYLON.StandardMaterial(
    `ceilingMat_${roomConfig.id}`,
    scene,
  );
  ceilingMaterial.diffuseColor = new BABYLON.Color3(...color);
  ceilingMaterial.specularColor = new BABYLON.Color3(0.05, 0.05, 0.05);
  ceilingMaterial.backFaceCulling = false;
  ceilingMaterial.alpha = 1.0;

  ceiling.material = ceilingMaterial;
  ceiling.position = position;
  ceiling.rotation.x = Math.PI / 2;

  return ceiling;
}

/**
 * Create a wall with slots and optional doorway
 */
function createWall(scene, roomConfig, wallConfig) {
  const { id, name, start, end, height, slots, doorway } = wallConfig;

  // Calculate wall parameters
  const startPos = new BABYLON.Vector3(...start);
  const endPos = new BABYLON.Vector3(...end);
  const wallVector = endPos.subtract(startPos);
  const wallLength = wallVector.length();
  const wallCenter = BABYLON.Vector3.Lerp(startPos, endPos, 0.5);
  wallCenter.y = height / 2;

  // Calculate wall rotation (facing inward)
  const wallDirection = wallVector.normalize();
  const angle = Math.atan2(wallDirection.x, wallDirection.z);

  // Create wall segments (if doorway exists, split the wall)
  const wallSegments = [];
  const wallMaterial = new BABYLON.StandardMaterial(
    `wallMat_${id}`,
    scene,
  );
  wallMaterial.diffuseColor = new BABYLON.Color3(...roomConfig.wall_color);
  wallMaterial.specularColor = new BABYLON.Color3(0.1, 0.1, 0.1);
  wallMaterial.backFaceCulling = false;

  if (doorway) {
    // Create wall segments around doorway
    const doorwayStart = doorway.offset - doorway.width / 2;
    const doorwayEnd = doorway.offset + doorway.width / 2;

    // Left segment
    if (doorwayStart > 0.1) {
      const leftSegment = createWallSegment(
        scene,
        `${id}_left`,
        startPos,
        wallDirection,
        doorwayStart,
        height,
        wallMaterial,
        angle,
      );
      wallSegments.push(leftSegment);
    }

    // Right segment
    if (doorwayEnd < wallLength - 0.1) {
      const rightStartPos = startPos.add(wallDirection.scale(doorwayEnd));
      const rightSegment = createWallSegment(
        scene,
        `${id}_right`,
        rightStartPos,
        wallDirection,
        wallLength - doorwayEnd,
        height,
        wallMaterial,
        angle,
      );
      wallSegments.push(rightSegment);
    }
  } else {
    // Single wall segment
    const segment = createWallSegment(
      scene,
      id,
      startPos,
      wallDirection,
      wallLength,
      height,
      wallMaterial,
      angle,
    );
    wallSegments.push(segment);
  }

  // Create slot positions
  const slotPositions = slots.map((slot) => {
    return calculateSlotPosition(
      startPos,
      wallDirection,
      angle,
      slot,
      height,
    );
  });

  return {
    wall: {
      id,
      name,
      segments: wallSegments,
      startPos,
      endPos,
      direction: wallDirection,
      angle,
      length: wallLength,
    },
    slots: slotPositions,
    doorway: doorway
      ? {
          id: `doorway_${id}`,
          position: startPos.add(wallDirection.scale(doorway.offset)),
          width: doorway.width,
          height: doorway.height,
        }
      : null,
  };
}

/**
 * Create a single wall segment mesh
 */
function createWallSegment(
  scene,
  id,
  startPos,
  direction,
  length,
  height,
  material,
  angle,
) {
  const segment = BABYLON.MeshBuilder.CreatePlane(
    `wall_${id}`,
    { width: length, height },
    scene,
  );

  const center = startPos.add(direction.scale(length / 2));
  center.y = height / 2;

  segment.position = center;
  segment.rotation.y = angle;
  segment.material = material;
  segment.receiveShadows = true;

  return segment;
}

/**
 * Calculate the 3D position for a media slot
 */
function calculateSlotPosition(startPos, wallDirection, wallAngle, slot, wallHeight) {
  const { id, offset, height, width, size_type } = slot;

  // Calculate position along the wall
  const slotCenter = startPos.add(wallDirection.scale(offset));
  slotCenter.y = height; // This is the height parameter from slot config

  // Calculate the normal (perpendicular to wall, pointing inward)
  const normal = new BABYLON.Vector3(
    Math.cos(wallAngle),
    0,
    -Math.sin(wallAngle),
  );

  // Position slightly in front of the wall
  const slotPosition = slotCenter.add(normal.scale(0.05));

  return {
    id,
    position: slotPosition,
    rotation: new BABYLON.Vector3(0, wallAngle, 0),
    width,
    height,
    size_type,
    wall_angle: wallAngle,
  };
}

/**
 * Create lighting for a room
 */
function createRoomLighting(scene, roomConfig) {
  const lights = [];
  const { position, dimensions } = roomConfig;
  const [x, y, z] = position;
  const { width, depth, height } = dimensions;

  // Ambient light for the room
  const ambient = new BABYLON.HemisphericLight(
    `ambient_${roomConfig.id}`,
    new BABYLON.Vector3(0, 1, 0),
    scene,
  );
  ambient.intensity = 0.5;
  ambient.diffuse = new BABYLON.Color3(1, 1, 0.98);
  lights.push(ambient);

  // Directional light from above
  const mainLight = new BABYLON.DirectionalLight(
    `main_${roomConfig.id}`,
    new BABYLON.Vector3(0.5, -1, 0.3),
    scene,
  );
  mainLight.position = new BABYLON.Vector3(x, y + height * 0.8, z);
  mainLight.intensity = 0.6;
  lights.push(mainLight);

  // Spotlights at corners
  const spotlightPositions = [
    new BABYLON.Vector3(x + width * 0.3, y + height * 0.9, z + depth * 0.3),
    new BABYLON.Vector3(x - width * 0.3, y + height * 0.9, z + depth * 0.3),
    new BABYLON.Vector3(x + width * 0.3, y + height * 0.9, z - depth * 0.3),
    new BABYLON.Vector3(x - width * 0.3, y + height * 0.9, z - depth * 0.3),
  ];

  spotlightPositions.forEach((pos, index) => {
    const spotlight = new BABYLON.SpotLight(
      `spot_${roomConfig.id}_${index}`,
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

  return lights;
}

/**
 * Get all slot positions from a gallery
 */
export function getSlotPositions(gallery) {
  return gallery.slots;
}

/**
 * Find a specific slot by ID
 */
export function findSlotById(gallery, slotId) {
  return gallery.slots.find((slot) => slot.id === slotId);
}

/**
 * Get slots filtered by size type
 */
export function getSlotsByType(gallery, sizeType) {
  return gallery.slots.filter((slot) => slot.size_type === sizeType);
}

/**
 * Map media to slots using the media_mapping from layout
 */
export function mapMediaToSlots(gallery, mediaMapping, mediaItems) {
  const mappedSlots = [];

  gallery.slots.forEach((slot) => {
    const mapping = mediaMapping[slot.id];
    if (mapping && mapping.media_id) {
      const mediaItem = mediaItems.find((item) => item.id === mapping.media_id);
      if (mediaItem) {
        mappedSlots.push({
          slot,
          media: mediaItem,
        });
      }
    }
  });

  return mappedSlots;
}

/**
 * Dispose of all gallery elements
 */
export function disposeGallery(gallery) {
  gallery.rooms.forEach((room) => {
    if (room.floor) room.floor.dispose();
    if (room.ceiling) room.ceiling.dispose();
    room.walls.forEach((wall) => {
      wall.segments.forEach((segment) => segment.dispose());
    });
  });

  gallery.lights.forEach((light) => light.dispose());

  console.log("Gallery disposed");
}

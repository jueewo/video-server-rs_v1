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
    shadowGenerators: [],
  };

  // Create each room
  layout.rooms.forEach((roomConfig) => {
    const room = createRoom(scene, roomConfig);
    gallery.rooms.push(room);
    gallery.walls.push(...room.walls);
    gallery.slots.push(...room.slots);
    gallery.doorways.push(...room.doorways);
    gallery.lights.push(...room.lights);
    if (room.shadowGenerator) {
      gallery.shadowGenerators.push(room.shadowGenerator);
    }
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
  const { id, name, floor_color, ceiling_color, height } = roomConfig;
  const dimensions = roomConfig.dimensions || { height: height || 4 };

  console.log(`Creating room: ${name}`);

  const room = {
    id,
    name,
    position: null, // Will be calculated from wall boundaries
    dimensions,
    floor: null,
    ceiling: null,
    walls: [],
    slots: [],
    doorways: [],
    lights: [],
    shadowGenerator: null,
  };

  // Calculate actual room center from wall boundaries
  let minX = Infinity,
    maxX = -Infinity;
  let minZ = Infinity,
    maxZ = -Infinity;

  roomConfig.walls.forEach((wall) => {
    const [startX, startY, startZ] = wall.start;
    const [endX, endY, endZ] = wall.end;
    minX = Math.min(minX, startX, endX);
    maxX = Math.max(maxX, startX, endX);
    minZ = Math.min(minZ, startZ, endZ);
    maxZ = Math.max(maxZ, startZ, endZ);
  });

  const actualCenterX = (minX + maxX) / 2;
  const actualCenterZ = (minZ + maxZ) / 2;
  const actualWidth = maxX - minX;
  const actualDepth = maxZ - minZ;
  const roomHeight = dimensions.height || height || 4;

  console.log(
    `  Actual bounds: X[${minX},${maxX}] Z[${minZ},${maxZ}] center=[${actualCenterX},${actualCenterZ}]`,
  );

  // Store calculated room center
  room.position = new BABYLON.Vector3(actualCenterX, 0, actualCenterZ);
  room.dimensions = {
    width: actualWidth,
    depth: actualDepth,
    height: roomHeight,
  };

  // Create floor at actual position
  room.floor = createFloor(
    scene,
    {
      ...roomConfig,
      dimensions: room.dimensions,
    },
    new BABYLON.Vector3(actualCenterX, 0, actualCenterZ),
    floor_color,
  );

  // Create ceiling at actual position
  room.ceiling = createCeiling(
    scene,
    {
      ...roomConfig,
      dimensions: room.dimensions,
    },
    new BABYLON.Vector3(actualCenterX, roomHeight, actualCenterZ),
    ceiling_color,
  );

  // Create walls with slots - pass calculated center
  roomConfig.walls.forEach((wallConfig) => {
    const wallData = createWall(
      scene,
      roomConfig,
      wallConfig,
      actualCenterX,
      actualCenterZ,
      roomHeight,
    );
    room.walls.push(wallData.wall);
    room.slots.push(...wallData.slots);
  });

  // Create lighting for the room with calculated dimensions
  const lightingResult = createRoomLighting(
    scene,
    roomConfig,
    actualCenterX,
    actualCenterZ,
    actualWidth,
    actualDepth,
    roomHeight,
  );
  room.lights = lightingResult.lights;
  room.shadowGenerator = lightingResult.shadowGenerator;

  // Make walls and floor receive shadows
  room.walls.forEach((wall) => {
    wall.segments.forEach((segment) => {
      segment.receiveShadows = true;
    });
  });
  if (room.floor) {
    room.floor.receiveShadows = true;
  }

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
  floor.renderingGroupId = 0; // Same group as walls

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
  ceiling.renderingGroupId = 0; // Same group as walls

  return ceiling;
}

/**
 * Create a wall segment with slots (no doorway splitting - walls are pre-split in JSON)
 */
function createWall(
  scene,
  roomConfig,
  wallConfig,
  roomCenterX,
  roomCenterZ,
  roomHeight,
) {
  const { id, name, start, end, slots } = wallConfig;
  const height = roomHeight;

  // Calculate wall parameters
  const startPos = new BABYLON.Vector3(...start);
  const endPos = new BABYLON.Vector3(...end);
  const wallVector = endPos.subtract(startPos);
  const wallLength = wallVector.length();
  const wallCenter = BABYLON.Vector3.Lerp(startPos, endPos, 0.5);
  wallCenter.y = height / 2;

  // Calculate wall rotation to face inward toward room center
  const wallDirection = wallVector.normalize();
  const roomCenter = new BABYLON.Vector3(roomCenterX, 0, roomCenterZ);

  // Vector from wall center to room center
  const toRoomCenter = roomCenter.subtract(wallCenter);
  toRoomCenter.y = 0; // Only consider horizontal direction

  // Calculate the normal direction (perpendicular to wall)
  // Cross product of wall direction with up vector gives the normal
  const normal = BABYLON.Vector3.Cross(
    wallDirection,
    new BABYLON.Vector3(0, 1, 0),
  );

  // Check if normal points toward room center, if not flip it
  const dotProduct = BABYLON.Vector3.Dot(normal, toRoomCenter);
  const facingDirection = dotProduct > 0 ? normal : normal.scale(-1);

  // Calculate angle from the facing direction
  const angle = Math.atan2(facingDirection.x, facingDirection.z);

  console.log(
    `Creating wall ${id}: start=[${startPos.x},${startPos.y},${startPos.z}] end=[${endPos.x},${endPos.y},${endPos.z}] center=[${wallCenter.x},${wallCenter.y},${wallCenter.z}] length=${wallLength} angle=${angle} (${(angle * 180) / Math.PI}°)`,
  );

  // Create single wall segment (walls are pre-split in JSON)
  const wallMaterial = new BABYLON.StandardMaterial(`wallMat_${id}`, scene);
  wallMaterial.diffuseColor = new BABYLON.Color3(...roomConfig.wall_color);
  wallMaterial.specularColor = new BABYLON.Color3(0.1, 0.1, 0.1);
  wallMaterial.backFaceCulling = false;
  wallMaterial.alpha = 1.0; // Fully opaque
  wallMaterial.transparencyMode = BABYLON.Material.MATERIAL_OPAQUE;

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
  const wallSegments = [segment];

  // Create slot positions - pass wall perpendicular direction so images face inward
  const slotPositions = slots.map((slot) => {
    return calculateSlotPosition(
      startPos,
      wallDirection,
      angle,
      slot,
      height,
      facingDirection,
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
  segment.renderingGroupId = 0; // Walls in group 0, images in group 1

  return segment;
}

/**
 * Calculate the 3D position for a media slot
 */
function calculateSlotPosition(
  startPos,
  wallDirection,
  wallAngle,
  slot,
  wallHeight,
  wallFacingDirection,
) {
  const { id, offset, height, width, size_type } = slot;

  // Calculate position along the wall
  const slotCenter = startPos.add(wallDirection.scale(offset));
  slotCenter.y = height; // This is the height parameter from slot config

  // Use wall perpendicular direction (always exactly perpendicular to wall surface)
  const facingDir = wallFacingDirection.clone();

  // Position in front of the wall toward room center
  const slotPosition = slotCenter.add(facingDir.scale(0.15));

  console.log(
    `📍 Slot ${id}: wall=[${slotCenter.x.toFixed(2)}, ${slotCenter.y.toFixed(2)}, ${slotCenter.z.toFixed(2)}], facingDir=[${facingDir.x.toFixed(2)}, ${facingDir.z.toFixed(2)}], final=[${slotPosition.x.toFixed(2)}, ${slotPosition.y.toFixed(2)}, ${slotPosition.z.toFixed(2)}]`,
  );

  return {
    id,
    position: slotPosition,
    rotation: new BABYLON.Vector3(0, wallAngle, 0),
    facingDirection: facingDir,
    width,
    height,
    size_type,
    wall_angle: wallAngle,
  };
}

/**
 * Create lighting for a room
 */
function createRoomLighting(
  scene,
  roomConfig,
  centerX,
  centerZ,
  width,
  depth,
  height,
) {
  const lights = [];

  const x = centerX;
  const y = 0;
  const z = centerZ;

  // Bright ambient light for overall illumination
  const ambient = new BABYLON.HemisphericLight(
    `ambient_${roomConfig.id}`,
    new BABYLON.Vector3(0, 1, 0),
    scene,
  );
  ambient.intensity = 0.7;
  ambient.diffuse = new BABYLON.Color3(1, 1, 0.98);
  lights.push(ambient);

  // Main directional light for shadow casting
  const mainLight = new BABYLON.DirectionalLight(
    `main_${roomConfig.id}`,
    new BABYLON.Vector3(0.3, -1, 0.2),
    scene,
  );
  mainLight.position = new BABYLON.Vector3(x, y + height * 0.9, z);
  mainLight.intensity = 0.8;

  // Enable shadows for this light
  const shadowGenerator = new BABYLON.ShadowGenerator(1024, mainLight);
  shadowGenerator.useBlurExponentialShadowMap = true;
  shadowGenerator.blurScale = 2;
  shadowGenerator.setDarkness(0.3);
  shadowGenerator.usePoissonSampling = true;

  lights.push(mainLight);

  return {
    lights,
    shadowGenerator,
  };
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
      // Find media item by ID and type (to distinguish between image ID 1 and video ID 1)
      const mediaItem = mediaItems.find((item) => {
        const idMatch = item.id === mapping.media_id;
        const typeMatch = mapping.type ? item.type === mapping.type : true;
        return idMatch && typeMatch;
      });
      if (mediaItem) {
        mappedSlots.push({
          slot,
          media: mediaItem,
        });
      } else {
        console.warn(`No media found for slot ${slot.id}: media_id=${mapping.media_id}, type=${mapping.type}`);
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

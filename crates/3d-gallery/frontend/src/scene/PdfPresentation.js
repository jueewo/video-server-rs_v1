/**
 * PdfPresentation.js
 *
 * Creates a 3D presentation frame for PDF documents in the gallery.
 * Renders PDF pages to a Babylon.js DynamicTexture using PDF.js.
 *
 * Uses the same TransformNode parent pattern as ImageFrame.js / VideoScreen.js:
 *   parent rotation = atan2(-facingDirection.x, -facingDirection.z)
 *   children placed in LOCAL space → local +X = viewer's right, always correct.
 */

import * as BABYLON from "@babylonjs/core";
import * as pdfjsLib from "pdfjs-dist";

// Point the worker to the CDN so esbuild doesn't bundle it
pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdn.jsdelivr.net/npm/pdfjs-dist@${pdfjsLib.version}/build/pdf.worker.min.js`;

/** Canvas resolution for the DynamicTexture (landscape 4:3) */
const TEX_W = 1024;
const TEX_H = 768;

function drawPlaceholder(ctx) {
  ctx.fillStyle = "#1a1a2e";
  ctx.fillRect(0, 0, TEX_W, TEX_H);
  ctx.fillStyle = "#e94560";
  ctx.font = "bold 80px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("PDF", TEX_W / 2, TEX_H / 2 - 40);
  ctx.fillStyle = "rgba(255,255,255,0.6)";
  ctx.font = "28px sans-serif";
  ctx.fillText("Loading…", TEX_W / 2, TEX_H / 2 + 50);
}

function drawPageIndicator(ctx, current, total) {
  const barH = 36;
  ctx.fillStyle = "rgba(0,0,0,0.55)";
  ctx.fillRect(0, TEX_H - barH, TEX_W, barH);
  ctx.fillStyle = "white";
  ctx.font = "20px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(`${current} / ${total}`, TEX_W / 2, TEX_H - barH / 2);
}

async function renderPage(pdfDoc, pageNum, ctx, texture) {
  const page = await pdfDoc.getPage(pageNum);
  const viewport = page.getViewport({ scale: 1.0 });
  const scale = Math.min(TEX_W / viewport.width, TEX_H / viewport.height) * 0.92;
  const scaled = page.getViewport({ scale });
  const offsetX = (TEX_W - scaled.width) / 2;
  const offsetY = (TEX_H - scaled.height) / 2;

  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, TEX_W, TEX_H);

  await page.render({
    canvasContext: ctx,
    viewport: scaled,
    transform: [1, 0, 0, 1, offsetX, offsetY],
  }).promise;

  drawPageIndicator(ctx, pageNum, pdfDoc.numPages);
  texture.update();
}

/**
 * Create an arrow overlay plane as a child of `parent` in LOCAL space.
 * local +X = viewer's right, local -X = viewer's left (same as frame border children).
 */
function createArrowOverlay(scene, direction, id, frameW, frameH, parent) {
  const arrowSize = Math.min(frameW, frameH) * 0.22;

  const canvas = document.createElement("canvas");
  canvas.width = 256;
  canvas.height = 256;
  const ctx = canvas.getContext("2d");

  ctx.fillStyle = "rgba(0,0,0,0.55)";
  ctx.beginPath();
  ctx.arc(128, 128, 110, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = "white";
  ctx.font = "bold 140px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(direction === "prev" ? "◄" : "►", 128, 135);

  const tex = new BABYLON.DynamicTexture(
    `pdfArrowTex_${direction}_${id}`,
    canvas,
    scene,
    false,
  );
  tex.hasAlpha = true;
  tex.update();

  const mat = new BABYLON.StandardMaterial(`pdfArrowMat_${direction}_${id}`, scene);
  mat.diffuseTexture = tex;
  mat.emissiveTexture = tex;
  mat.opacityTexture = tex;
  mat.backFaceCulling = false;
  mat.useAlphaFromDiffuseTexture = true;
  mat.transparencyMode = BABYLON.Material.MATERIAL_ALPHABLEND;

  const plane = BABYLON.MeshBuilder.CreatePlane(
    `pdfArrow_${direction}_${id}`,
    { width: arrowSize, height: arrowSize },
    scene,
  );
  plane.material = mat;
  plane.isPickable = true;
  plane.parent = parent;

  // Place in LOCAL space: -X = left (prev), +X = right (next)
  const offsetSign = direction === "prev" ? -1 : 1;
  plane.position.x = offsetSign * (frameW / 2 + arrowSize / 2 + 0.05);
  plane.position.z = -0.01; // Same depth as the frame plane

  return plane;
}

/**
 * Create a 3D PDF presentation frame.
 *
 * @param {BABYLON.Scene} scene
 * @param {object} media   { id, title, url, description, tags }
 * @param {object} options
 * @param {BABYLON.Vector3} options.position
 * @param {BABYLON.Vector3} options.facingDirection  wall normal toward room center
 * @param {number} options.width  world units
 */
/**
 * Create a 4-piece frame border around the PDF plane (same as ImageFrame.js).
 * All pieces are children of `parent` so they rotate with the wall.
 */
function createFrameBorder(scene, width, height, thickness, id, parent) {
  const depth = thickness / 2;
  const mat = new BABYLON.StandardMaterial(`pdfFrameMat_${id}`, scene);
  mat.diffuseColor = new BABYLON.Color3(0.05, 0.05, 0.05); // Black, like video screens
  mat.specularColor = new BABYLON.Color3(0.3, 0.25, 0.2);

  const pieces = [
    // Top
    { w: width + thickness * 2, h: thickness, d: depth, x: 0, y: height / 2 + thickness / 2 },
    // Bottom
    { w: width + thickness * 2, h: thickness, d: depth, x: 0, y: -(height / 2 + thickness / 2) },
    // Left
    { w: thickness, h: height, d: depth, x: -(width / 2 + thickness / 2), y: 0 },
    // Right
    { w: thickness, h: height, d: depth, x: width / 2 + thickness / 2, y: 0 },
  ];

  pieces.forEach((p, i) => {
    const box = BABYLON.MeshBuilder.CreateBox(`pdfBorder_${id}_${i}`, { width: p.w, height: p.h, depth: p.d }, scene);
    box.parent = parent;
    box.position.x = p.x;
    box.position.y = p.y;
    box.position.z = -depth / 2; // Same depth as image frame border
    box.material = mat;
    box.isPickable = false;
    box.renderingGroupId = 0;
  });
}

export function createPdfPresentation(scene, media, options) {
  const { position, facingDirection, width = 2.5, frameThickness = 0.12 } = options;

  const aspectRatio = TEX_W / TEX_H; // 4:3
  const height = width / aspectRatio;
  const id = media.id;

  // --- Parent TransformNode (same pattern as ImageFrame / VideoScreen) ---
  const parent = new BABYLON.TransformNode(`pdfPresentation_${id}`, scene);
  parent.position = position.clone();

  // Rotation: local -Z faces the room (toward viewer), same formula as ImageFrame.js
  if (facingDirection) {
    const rotY = Math.atan2(-facingDirection.x, -facingDirection.z);
    parent.rotation = new BABYLON.Vector3(0, rotY, 0);
  }

  // --- Frame plane (child, in local space) ---
  const framePlane = BABYLON.MeshBuilder.CreatePlane(
    `pdfFrame_${id}`,
    { width, height },
    scene,
  );
  framePlane.parent = parent;
  framePlane.position.z = -0.01; // Slightly in front of wall

  // Create canvas externally (same pattern as VideoScreen play button overlay)
  // DynamicTexture created from an existing canvas is more reliable than getContext()
  const texCanvas = document.createElement("canvas");
  texCanvas.width = TEX_W;
  texCanvas.height = TEX_H;
  const texCtx = texCanvas.getContext("2d");
  drawPlaceholder(texCtx);

  const texture = new BABYLON.DynamicTexture(
    `pdfTex_${id}`,
    texCanvas,
    scene,
    false,
  );
  texture.hasAlpha = false;
  // No vScale flip — DynamicTexture canvas coordinates already match WebGL UV space
  texture.update();

  const mat = new BABYLON.StandardMaterial(`pdfMat_${id}`, scene);
  mat.diffuseTexture = texture;
  mat.emissiveTexture = texture;
  mat.backFaceCulling = false;
  framePlane.material = mat;

  framePlane.metadata = {
    type: "document",
    id: media.id,
    title: media.title,
    description: media.description,
    url: media.url,
    media_type: media.media_type,
    tags: media.tags || [],
  };
  framePlane.isPickable = true;

  // --- Frame border (same as ImageFrame.js) ---
  createFrameBorder(scene, width, height, frameThickness, id, parent);

  // --- Arrow overlays (children, positioned in local space) ---
  const prevArrow = createArrowOverlay(scene, "prev", id, width, height, parent);
  const nextArrow = createArrowOverlay(scene, "next", id, width, height, parent);

  // State
  let pdfDoc = null;
  let currentPage = 1;

  async function goToPage(pageNum) {
    if (!pdfDoc) return;
    currentPage = Math.max(1, Math.min(pageNum, pdfDoc.numPages));
    await renderPage(pdfDoc, currentPage, texCtx, texture);
  }

  prevArrow.actionManager = new BABYLON.ActionManager(scene);
  prevArrow.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPickDownTrigger,
      () => goToPage(currentPage - 1),
    ),
  );

  nextArrow.actionManager = new BABYLON.ActionManager(scene);
  nextArrow.actionManager.registerAction(
    new BABYLON.ExecuteCodeAction(
      BABYLON.ActionManager.OnPickDownTrigger,
      () => goToPage(currentPage + 1),
    ),
  );

  // Load PDF
  pdfjsLib
    .getDocument(media.url)
    .promise.then(async (doc) => {
      pdfDoc = doc;
      console.log(`PDF loaded: ${media.title} (${doc.numPages} pages)`);
      await renderPage(pdfDoc, 1, texCtx, texture);
    })
    .catch((err) => {
      console.error("Failed to load PDF for 3D gallery:", err);
      texCtx.fillStyle = "#1a1a2e";
      texCtx.fillRect(0, 0, TEX_W, TEX_H);
      texCtx.fillStyle = "#e94560";
      texCtx.font = "bold 48px sans-serif";
      texCtx.textAlign = "center";
      texCtx.textBaseline = "middle";
      texCtx.fillText("Failed to load PDF", TEX_W / 2, TEX_H / 2);
      texture.update();
    });

  function dispose() {
    parent.dispose(); // disposes all children too
    texture.dispose();
    mat.dispose();
  }

  return { framePlane, prevArrow, nextArrow, dispose };
}

/**
 * Map grid layout + hit-testing (G03).
 *
 * Shared between the Canvas renderer and click handling so tile coordinates
 * stay consistent with what the user sees.
 */

import { type WorldState } from "./world.js";

export const LEGEND_HEIGHT = 44;

export interface GridLayout {
  cols: number;
  rows: number;
  cell: number;
  offsetX: number;
  offsetY: number;
  gridW: number;
  gridH: number;
  mapHeight: number;
}

/** Compute the centered grid layout for the current canvas + world. */
export function computeGridLayout(
  canvasWidth: number,
  canvasHeight: number,
  world: WorldState,
): GridLayout | null {
  if (!world.isReady()) {
    return null;
  }
  const cols = world.mapWidth;
  const rows = world.mapHeight;
  const mapHeight = Math.max(1, canvasHeight - LEGEND_HEIGHT);
  const cell = Math.max(
    1,
    Math.floor(Math.min(canvasWidth / cols, mapHeight / rows)),
  );
  const gridW = cell * cols;
  const gridH = cell * rows;
  const offsetX = Math.floor((canvasWidth - gridW) / 2);
  const offsetY = Math.floor((mapHeight - gridH) / 2);
  return { cols, rows, cell, offsetX, offsetY, gridW, gridH, mapHeight };
}

/**
 * Map a canvas-pixel point to a tile coordinate, or `null` if outside the grid
 * (or over the legend strip).
 */
export function hitTestTile(
  canvasX: number,
  canvasY: number,
  layout: GridLayout,
): { x: number; y: number } | null {
  const { offsetX, offsetY, cell, cols, rows, gridW, gridH } = layout;
  if (
    canvasX < offsetX ||
    canvasY < offsetY ||
    canvasX >= offsetX + gridW ||
    canvasY >= offsetY + gridH
  ) {
    return null;
  }
  const x = Math.floor((canvasX - offsetX) / cell);
  const y = Math.floor((canvasY - offsetY) / cell);
  if (x < 0 || y < 0 || x >= cols || y >= rows) {
    return null;
  }
  return { x, y };
}

/**
 * Map a canvas-pixel point to the nearest player icon, or `null`.
 * Prefers players over bare tiles so AQ19 click-player works even when the
 * icon sits inside a resource cell (G04).
 */
export function hitTestPlayer(
  canvasX: number,
  canvasY: number,
  layout: GridLayout,
  world: WorldState,
): number | null {
  const { offsetX, offsetY, cell } = layout;
  const radius = Math.max(3, Math.floor(cell / 4.5));
  // Slightly larger hit target than the drawn icon.
  const hitR = radius * 1.6;
  let bestId: number | null = null;
  let bestDist = hitR * hitR;
  for (const player of world.allPlayers()) {
    const cx = offsetX + player.x * cell + cell / 2;
    const cy = offsetY + player.y * cell + cell / 2;
    const dx = canvasX - cx;
    const dy = canvasY - cy;
    const d2 = dx * dx + dy * dy;
    if (d2 <= bestDist) {
      bestDist = d2;
      bestId = player.id;
    }
  }
  return bestId;
}

/**
 * Convert a mouse event on `canvas` into canvas-buffer coordinates (accounts
 * for CSS scaling / devicePixelRatio).
 */
export function eventToCanvasPoint(
  canvas: HTMLCanvasElement,
  clientX: number,
  clientY: number,
): { x: number; y: number } {
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / Math.max(1, rect.width);
  const scaleY = canvas.height / Math.max(1, rect.height);
  return {
    x: (clientX - rect.left) * scaleX,
    y: (clientY - rect.top) * scaleY,
  };
}

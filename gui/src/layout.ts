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

/**
 * Canvas renderer (G01).
 *
 * Draws the toroidal grid and marks tiles that currently hold resources. G01
 * deliberately keeps marks minimal (a presence dot) — per-resource icons are
 * G02, tile/player detail overlays are G03/G04. No game engine: plain
 * HTML5 Canvas 2D.
 */

import { tileHasResources } from "./protocol.js";
import { type WorldState } from "./world.js";

const BACKGROUND = "#0c1b12";
const GRID_LINE = "#1f3b29";
const RESOURCE_DOT = "#57d98a";
const EMPTY_TEXT = "#6a8c76";

export class CanvasRenderer {
  private readonly ctx: CanvasRenderingContext2D;

  constructor(private readonly canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext("2d");
    if (ctx === null) {
      throw new Error("2D canvas context unavailable");
    }
    this.ctx = ctx;
  }

  /** Redraw the whole frame for the current world state. */
  render(world: WorldState): void {
    const { width, height } = this.canvas;
    const ctx = this.ctx;

    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, width, height);

    if (!world.isReady()) {
      this.drawPlaceholder("waiting for map size (msz)…");
      return;
    }

    const cols = world.mapWidth;
    const rows = world.mapHeight;
    const cell = Math.max(
      1,
      Math.floor(Math.min(width / cols, height / rows)),
    );
    const gridW = cell * cols;
    const gridH = cell * rows;
    const offsetX = Math.floor((width - gridW) / 2);
    const offsetY = Math.floor((height - gridH) / 2);

    this.drawGrid(offsetX, offsetY, cols, rows, cell);
    this.drawResources(world, offsetX, offsetY, cols, rows, cell);
  }

  private drawGrid(
    offsetX: number,
    offsetY: number,
    cols: number,
    rows: number,
    cell: number,
  ): void {
    const ctx = this.ctx;
    ctx.strokeStyle = GRID_LINE;
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let c = 0; c <= cols; c += 1) {
      const x = offsetX + c * cell + 0.5;
      ctx.moveTo(x, offsetY);
      ctx.lineTo(x, offsetY + rows * cell);
    }
    for (let r = 0; r <= rows; r += 1) {
      const y = offsetY + r * cell + 0.5;
      ctx.moveTo(offsetX, y);
      ctx.lineTo(offsetX + cols * cell, y);
    }
    ctx.stroke();
  }

  private drawResources(
    world: WorldState,
    offsetX: number,
    offsetY: number,
    cols: number,
    rows: number,
    cell: number,
  ): void {
    const ctx = this.ctx;
    const radius = Math.max(1, Math.floor(cell / 6));
    ctx.fillStyle = RESOURCE_DOT;
    for (let y = 0; y < rows; y += 1) {
      for (let x = 0; x < cols; x += 1) {
        const tile = world.tileAt(x, y);
        if (tile !== null && tileHasResources(tile)) {
          const cx = offsetX + x * cell + cell / 2;
          const cy = offsetY + y * cell + cell / 2;
          ctx.beginPath();
          ctx.arc(cx, cy, radius, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }
  }

  private drawPlaceholder(text: string): void {
    const ctx = this.ctx;
    ctx.fillStyle = EMPTY_TEXT;
    ctx.font = "16px system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(text, this.canvas.width / 2, this.canvas.height / 2);
  }
}

/**
 * Canvas renderer (G01 / G02).
 *
 * Draws the toroidal grid plus distinct icons for food, all six stone types,
 * and players (RQ21 / AQ18 / AQ28). A legend strip keeps stone types
 * distinguishable. No game engine: plain HTML5 Canvas 2D.
 */

import { presentResources } from "./protocol.js";
import {
  ICON_SPECS,
  drawIcon,
  iconSlots,
  resourceIconOrder,
} from "./icons.js";
import { type WorldState } from "./world.js";

const BACKGROUND = "#0c1b12";
const GRID_LINE = "#1f3b29";
const EMPTY_TEXT = "#6a8c76";
const LEGEND_BG = "#081109";
const LEGEND_TEXT = "#cfe8d8";
const LEGEND_HEIGHT = 44;

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
      this.drawLegend(width, height);
      return;
    }

    const cols = world.mapWidth;
    const rows = world.mapHeight;
    const mapHeight = Math.max(1, height - LEGEND_HEIGHT);
    const cell = Math.max(
      1,
      Math.floor(Math.min(width / cols, mapHeight / rows)),
    );
    const gridW = cell * cols;
    const gridH = cell * rows;
    const offsetX = Math.floor((width - gridW) / 2);
    const offsetY = Math.floor((mapHeight - gridH) / 2);

    this.drawGrid(offsetX, offsetY, cols, rows, cell);
    this.drawTileIcons(world, offsetX, offsetY, cols, rows, cell);
    this.drawPlayers(world, offsetX, offsetY, cell);
    this.drawLegend(width, height);
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

  private drawTileIcons(
    world: WorldState,
    offsetX: number,
    offsetY: number,
    cols: number,
    rows: number,
    cell: number,
  ): void {
    const radius = Math.max(2, Math.floor(cell / 7));
    for (let y = 0; y < rows; y += 1) {
      for (let x = 0; x < cols; x += 1) {
        const tile = world.tileAt(x, y);
        if (tile === null) {
          continue;
        }
        const kinds = presentResources(tile);
        if (kinds.length === 0) {
          continue;
        }
        const slots = iconSlots(kinds.length, cell);
        for (let i = 0; i < kinds.length; i += 1) {
          const slot = slots[i];
          if (slot === undefined) {
            continue;
          }
          const spec = ICON_SPECS[kinds[i]];
          drawIcon(
            this.ctx,
            spec,
            offsetX + x * cell + slot.x,
            offsetY + y * cell + slot.y,
            radius,
          );
        }
      }
    }
  }

  private drawPlayers(
    world: WorldState,
    offsetX: number,
    offsetY: number,
    cell: number,
  ): void {
    const radius = Math.max(3, Math.floor(cell / 4.5));
    const spec = ICON_SPECS.player;
    for (const player of world.allPlayers()) {
      const cx = offsetX + player.x * cell + cell / 2;
      const cy = offsetY + player.y * cell + cell / 2;
      drawIcon(this.ctx, spec, cx, cy, radius);
    }
  }

  private drawLegend(canvasW: number, canvasH: number): void {
    const ctx = this.ctx;
    const top = canvasH - LEGEND_HEIGHT;
    ctx.fillStyle = LEGEND_BG;
    ctx.fillRect(0, top, canvasW, LEGEND_HEIGHT);
    ctx.strokeStyle = GRID_LINE;
    ctx.beginPath();
    ctx.moveTo(0, top + 0.5);
    ctx.lineTo(canvasW, top + 0.5);
    ctx.stroke();

    const entries = [...resourceIconOrder(), "player" as const];
    const slotW = canvasW / entries.length;
    const cy = top + LEGEND_HEIGHT / 2;
    ctx.font = "11px system-ui, sans-serif";
    ctx.textBaseline = "middle";
    ctx.textAlign = "left";

    for (let i = 0; i < entries.length; i += 1) {
      const kind = entries[i];
      const spec = ICON_SPECS[kind];
      const x0 = i * slotW;
      drawIcon(ctx, spec, x0 + 12, cy, 6);
      ctx.fillStyle = LEGEND_TEXT;
      ctx.fillText(spec.label, x0 + 22, cy);
    }
  }

  private drawPlaceholder(text: string): void {
    const ctx = this.ctx;
    ctx.fillStyle = EMPTY_TEXT;
    ctx.font = "16px system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(
      text,
      this.canvas.width / 2,
      (this.canvas.height - LEGEND_HEIGHT) / 2,
    );
  }
}

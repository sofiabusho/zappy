/**
 * Canvas renderer (G01 / G02 / G03 layout shared with hit-testing).
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
import { LEGEND_HEIGHT, computeGridLayout } from "./layout.js";
import {
  soundAlpha,
  soundArrivalVector,
  soundRippleRadius,
  type SoundEvent,
} from "./sound.js";
import { type WorldState } from "./world.js";

const BACKGROUND = "#0c1b12";
const GRID_LINE = "#1f3b29";
const EMPTY_TEXT = "#6a8c76";
const LEGEND_BG = "#081109";
const LEGEND_TEXT = "#cfe8d8";
const SELECT_STROKE = "#cfe8d8";
const SOUND_STROKE = "#7ec8ff";
const SOUND_FILL = "rgba(126, 200, 255, 0.15)";

export class CanvasRenderer {
  private readonly ctx: CanvasRenderingContext2D;
  private selected: { x: number; y: number } | null = null;
  private selectedPlayerId: number | null = null;

  constructor(private readonly canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext("2d");
    if (ctx === null) {
      throw new Error("2D canvas context unavailable");
    }
    this.ctx = ctx;
  }

  /** Highlight the inspected square (G03); `null` clears. */
  setSelectedTile(tile: { x: number; y: number } | null): void {
    this.selected = tile;
  }

  /** Highlight the inspected player (G04); `null` clears. */
  setSelectedPlayer(id: number | null): void {
    this.selectedPlayerId = id;
  }

  /** Redraw the whole frame for the current world state. */
  render(world: WorldState, now = performance.now()): void {
    const { width, height } = this.canvas;
    const ctx = this.ctx;

    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, width, height);

    if (!world.isReady()) {
      this.drawPlaceholder("waiting for map size (msz)…");
      this.drawLegend(width, height);
      return;
    }

    const layout = computeGridLayout(width, height, world);
    if (layout === null) {
      return;
    }
    const { cols, rows, cell, offsetX, offsetY } = layout;

    this.drawGrid(offsetX, offsetY, cols, rows, cell);
    this.drawTileIcons(world, offsetX, offsetY, cols, rows, cell);
    this.drawPlayers(world, offsetX, offsetY, cell);
    this.drawSounds(world, offsetX, offsetY, cell, now);
    this.drawSelection(offsetX, offsetY, cell);
    this.drawPlayerSelection(world, offsetX, offsetY, cell);
    this.drawLegend(width, height);
  }

  private drawSounds(
    world: WorldState,
    offsetX: number,
    offsetY: number,
    cell: number,
    now: number,
  ): void {
    const events = world.activeSounds(now);
    for (const ev of events) {
      const alpha = soundAlpha(ev.bornAt, now);
      if (alpha <= 0) {
        continue;
      }
      if (ev.kind === "emit") {
        this.drawEmitRipple(ev, offsetX, offsetY, cell, now, alpha);
      } else if (ev.k !== null && ev.toId !== null) {
        this.drawHearArrow(world, ev, offsetX, offsetY, cell, alpha);
      }
    }
  }

  private drawEmitRipple(
    ev: SoundEvent,
    offsetX: number,
    offsetY: number,
    cell: number,
    now: number,
    alpha: number,
  ): void {
    const ctx = this.ctx;
    const cx = offsetX + ev.x * cell + cell / 2;
    const cy = offsetY + ev.y * cell + cell / 2;
    const r = soundRippleRadius(ev.bornAt, now, cell);
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = SOUND_STROKE;
    ctx.fillStyle = SOUND_FILL;
    ctx.lineWidth = Math.max(1.5, cell / 18);
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    // Inner pulse
    ctx.beginPath();
    ctx.arc(cx, cy, Math.max(2, r * 0.35), 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  }

  private drawHearArrow(
    world: WorldState,
    ev: SoundEvent,
    offsetX: number,
    offsetY: number,
    cell: number,
    alpha: number,
  ): void {
    if (ev.toId === null || ev.k === null) {
      return;
    }
    const player = world.playerById(ev.toId);
    if (player === null) {
      return;
    }
    const ctx = this.ctx;
    const cx = offsetX + player.x * cell + cell / 2;
    const cy = offsetY + player.y * cell + cell / 2;
    const vec = soundArrivalVector(ev.k, player.orientation);
    const len = cell * 0.42;

    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = SOUND_STROKE;
    ctx.fillStyle = SOUND_STROKE;
    ctx.lineWidth = Math.max(1.5, cell / 20);

    if (ev.k === 0) {
      // Same-tile sound: concentric rings on the listener.
      ctx.beginPath();
      ctx.arc(cx, cy, cell * 0.28, 0, Math.PI * 2);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(cx, cy, cell * 0.18, 0, Math.PI * 2);
      ctx.stroke();
    } else {
      const tipX = cx + vec.dx * len;
      const tipY = cy + vec.dy * len;
      const baseX = cx + vec.dx * len * 0.25;
      const baseY = cy + vec.dy * len * 0.25;
      ctx.beginPath();
      ctx.moveTo(baseX, baseY);
      ctx.lineTo(tipX, tipY);
      ctx.stroke();
      // Arrow head
      const orthoX = -vec.dy;
      const orthoY = vec.dx;
      const head = cell * 0.12;
      ctx.beginPath();
      ctx.moveTo(tipX, tipY);
      ctx.lineTo(tipX - vec.dx * head + orthoX * head * 0.6, tipY - vec.dy * head + orthoY * head * 0.6);
      ctx.lineTo(tipX - vec.dx * head - orthoX * head * 0.6, tipY - vec.dy * head - orthoY * head * 0.6);
      ctx.closePath();
      ctx.fill();
    }

    // Label K next to the player.
    ctx.font = `${Math.max(10, Math.floor(cell / 3.5))}px system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "bottom";
    ctx.fillText(`K=${ev.k}`, cx, cy - cell * 0.35);
    ctx.restore();
  }

  private drawPlayerSelection(
    world: WorldState,
    offsetX: number,
    offsetY: number,
    cell: number,
  ): void {
    if (this.selectedPlayerId === null) {
      return;
    }
    const player = world.playerById(this.selectedPlayerId);
    if (player === null) {
      return;
    }
    const ctx = this.ctx;
    const cx = offsetX + player.x * cell + cell / 2;
    const cy = offsetY + player.y * cell + cell / 2;
    const r = Math.max(4, Math.floor(cell / 3.2));
    ctx.strokeStyle = SELECT_STROKE;
    ctx.lineWidth = Math.max(2, Math.floor(cell / 16));
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.stroke();
  }

  private drawSelection(
    offsetX: number,
    offsetY: number,
    cell: number,
  ): void {
    if (this.selected === null) {
      return;
    }
    const ctx = this.ctx;
    const x = offsetX + this.selected.x * cell;
    const y = offsetY + this.selected.y * cell;
    ctx.strokeStyle = SELECT_STROKE;
    ctx.lineWidth = Math.max(2, Math.floor(cell / 16));
    ctx.strokeRect(x + 1, y + 1, cell - 2, cell - 2);
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

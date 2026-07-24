/**
 * Map icon catalog (G02).
 *
 * Distinct 2D Canvas glyphs for food, the six subject stone types (AQ28), and
 * players (AQ18). No spritesheets / game engine — pure path drawing.
 */

import { RESOURCE_NAMES, type ResourceName } from "./protocol.js";

/** The six stone types from `docs/raw/requirements.md` (AQ28). */
export const STONE_NAMES = [
  "jade",
  "peridot",
  "amber",
  "amethyst",
  "garnet",
  "ammolite",
] as const;

export type StoneName = (typeof STONE_NAMES)[number];

export type IconKind = ResourceName | "player";

export type IconShape =
  | "circle"
  | "diamond"
  | "triangle"
  | "square"
  | "hex"
  | "chevron";

export interface IconSpec {
  kind: IconKind;
  label: string;
  color: string;
  shape: IconShape;
}

/** Visual specs keyed by icon kind — colors/shapes stay distinguishable. */
export const ICON_SPECS: Record<IconKind, IconSpec> = {
  food: { kind: "food", label: "food", color: "#7dff9a", shape: "circle" },
  jade: { kind: "jade", label: "jade", color: "#2ecc71", shape: "diamond" },
  peridot: {
    kind: "peridot",
    label: "peridot",
    color: "#a8e10c",
    shape: "triangle",
  },
  amber: { kind: "amber", label: "amber", color: "#f0a202", shape: "square" },
  amethyst: {
    kind: "amethyst",
    label: "amethyst",
    color: "#9b5de5",
    shape: "hex",
  },
  garnet: { kind: "garnet", label: "garnet", color: "#e63946", shape: "diamond" },
  ammolite: {
    kind: "ammolite",
    label: "ammolite",
    color: "#00bbf9",
    shape: "circle",
  },
  player: {
    kind: "player",
    label: "player",
    color: "#ffe66d",
    shape: "chevron",
  },
};

/** Resource icons in canonical draw order (food then stones). */
export function resourceIconOrder(): ResourceName[] {
  return [...RESOURCE_NAMES];
}

/** Assert the catalog covers food + all six stones + player (AQ18 / AQ28). */
export function iconCatalogComplete(): boolean {
  if (STONE_NAMES.length !== 6) {
    return false;
  }
  for (const stone of STONE_NAMES) {
    if (ICON_SPECS[stone] === undefined) {
      return false;
    }
  }
  return (
    ICON_SPECS.food !== undefined &&
    ICON_SPECS.player !== undefined &&
    RESOURCE_NAMES.includes("food")
  );
}

/**
 * Lay out up to `count` icon centers inside a cell of size `cell`, returning
 * offsets relative to the cell's top-left corner.
 */
export function iconSlots(
  count: number,
  cell: number,
): Array<{ x: number; y: number }> {
  if (count <= 0) {
    return [];
  }
  const pad = Math.max(2, Math.floor(cell * 0.18));
  const usable = Math.max(1, cell - pad * 2);
  const cols = Math.ceil(Math.sqrt(count));
  const rows = Math.ceil(count / cols);
  const stepX = usable / cols;
  const stepY = usable / rows;
  const slots: Array<{ x: number; y: number }> = [];
  for (let i = 0; i < count; i += 1) {
    const c = i % cols;
    const r = Math.floor(i / cols);
    slots.push({
      x: pad + stepX * (c + 0.5),
      y: pad + stepY * (r + 0.5),
    });
  }
  return slots;
}

/** Draw one icon centered at `(cx, cy)` with approximate radius `r`. */
export function drawIcon(
  ctx: CanvasRenderingContext2D,
  spec: IconSpec,
  cx: number,
  cy: number,
  r: number,
): void {
  ctx.save();
  ctx.fillStyle = spec.color;
  ctx.strokeStyle = "#06140c";
  ctx.lineWidth = Math.max(1, r * 0.15);

  switch (spec.shape) {
    case "circle":
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      break;

    case "square": {
      const s = r * 1.6;
      ctx.beginPath();
      ctx.rect(cx - s / 2, cy - s / 2, s, s);
      ctx.fill();
      ctx.stroke();
      break;
    }

    case "diamond":
      ctx.beginPath();
      ctx.moveTo(cx, cy - r);
      ctx.lineTo(cx + r, cy);
      ctx.lineTo(cx, cy + r);
      ctx.lineTo(cx - r, cy);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
      break;

    case "triangle":
      ctx.beginPath();
      ctx.moveTo(cx, cy - r);
      ctx.lineTo(cx + r, cy + r * 0.85);
      ctx.lineTo(cx - r, cy + r * 0.85);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
      break;

    case "hex": {
      ctx.beginPath();
      for (let i = 0; i < 6; i += 1) {
        const a = (Math.PI / 3) * i - Math.PI / 2;
        const x = cx + Math.cos(a) * r;
        const y = cy + Math.sin(a) * r;
        if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      }
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
      break;
    }

    case "chevron":
      // Player: upward chevron (facing) with a small body dot.
      ctx.beginPath();
      ctx.moveTo(cx, cy - r);
      ctx.lineTo(cx + r * 0.85, cy + r * 0.35);
      ctx.lineTo(cx, cy + r * 0.05);
      ctx.lineTo(cx - r * 0.85, cy + r * 0.35);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(cx, cy + r * 0.55, r * 0.28, 0, Math.PI * 2);
      ctx.fill();
      break;
  }

  ctx.restore();
}

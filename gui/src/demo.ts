/**
 * Offline demo stream (G01 / G02).
 *
 * Produces a deterministic sequence of server → GUI protocol lines so the map
 * renders with food, all six stone types, and players visible with no server
 * attached. Used when no `?ws=` bridge URL is supplied.
 */

import { RESOURCE_NAMES } from "./protocol.js";
import { STONE_NAMES } from "./icons.js";

/**
 * Build a demo line sequence for a `width` × `height` map.
 *
 * Guarantees (AQ18 / AQ28):
 * - every stone type appears on at least one tile
 * - food appears on at least one tile
 * - at least one `pnw` player is present
 */
export function demoStream(width = 12, height = 12): string[] {
  const lines: string[] = [`msz ${width} ${height}`, "tna alpha", "tna beta"];

  // Explicit showcase tiles so all six stones + food are unambiguously present.
  const showcase: Array<{ x: number; y: number; counts: number[] }> = [
    { x: 1, y: 1, counts: [1, 0, 0, 0, 0, 0, 0] }, // food
    { x: 3, y: 1, counts: [0, 1, 0, 0, 0, 0, 0] }, // jade
    { x: 5, y: 1, counts: [0, 0, 1, 0, 0, 0, 0] }, // peridot
    { x: 7, y: 1, counts: [0, 0, 0, 1, 0, 0, 0] }, // amber
    { x: 9, y: 1, counts: [0, 0, 0, 0, 1, 0, 0] }, // amethyst
    { x: 2, y: 3, counts: [0, 0, 0, 0, 0, 1, 0] }, // garnet
    { x: 4, y: 3, counts: [0, 0, 0, 0, 0, 0, 1] }, // ammolite
    { x: 6, y: 3, counts: [1, 1, 0, 1, 0, 0, 0] }, // mixed (≤3 stone types)
  ];

  for (const tile of showcase) {
    if (tile.x < width && tile.y < height) {
      lines.push(`bct ${tile.x} ${tile.y} ${tile.counts.join(" ")}`);
    }
  }

  // Sparse scatter elsewhere (still respects per-tile emptiness).
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      if (showcase.some((t) => t.x === x && t.y === y)) {
        continue;
      }
      const counts = RESOURCE_NAMES.map((_, i) =>
        (x * 7 + y * 13 + i * 5) % 19 === 0 ? 1 : 0,
      );
      // Cap to at most one food and three stone types (subject rules).
      if (counts[0]! > 1) {
        counts[0] = 1;
      }
      let stoneKinds = 0;
      for (let i = 1; i < counts.length; i += 1) {
        if (counts[i]! > 0) {
          stoneKinds += 1;
          if (stoneKinds > 3) {
            counts[i] = 0;
          }
        }
      }
      if (counts.some((n) => n > 0)) {
        lines.push(`bct ${x} ${y} ${counts.join(" ")}`);
      }
    }
  }

  // Players (AQ18).
  lines.push("pnw #1 6 6 1 1 alpha");
  lines.push("pnw #2 8 4 2 2 beta");
  lines.push("ppo #1 6 7 3");

  return lines;
}

/** Resource kinds that appear at least once in a demo `bct` stream. */
export function demoResourceCoverage(lines: string[]): Set<string> {
  const seen = new Set<string>();
  for (const line of lines) {
    if (!line.startsWith("bct ")) {
      continue;
    }
    const parts = line.split(/\s+/);
    for (let i = 0; i < RESOURCE_NAMES.length; i += 1) {
      const n = Number.parseInt(parts[3 + i] ?? "0", 10);
      if (n > 0) {
        seen.add(RESOURCE_NAMES[i]);
      }
    }
  }
  return seen;
}

/** True when the demo covers food + all six stones + at least one player. */
export function demoCoversIcons(lines: string[]): boolean {
  const resources = demoResourceCoverage(lines);
  if (!resources.has("food")) {
    return false;
  }
  for (const stone of STONE_NAMES) {
    if (!resources.has(stone)) {
      return false;
    }
  }
  return lines.some((line) => line.startsWith("pnw "));
}

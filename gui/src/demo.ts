/**
 * Offline demo stream (G01).
 *
 * Produces a deterministic sequence of server → GUI protocol lines so the map
 * renders (empty grid first, then resources filling in) with no server
 * attached. This is what the GUI falls back to when no `?ws=` bridge URL is
 * supplied, and it doubles as a fixture for the manual render checklist.
 */

import { RESOURCE_NAMES } from "./protocol.js";

/**
 * Build a demo line sequence for a `width` × `height` map. The first line is
 * `msz` (grid becomes drawable); each following `bct` line seeds one tile so
 * the renderer visibly fills the world over time.
 */
export function demoStream(width = 12, height = 12): string[] {
  const lines: string[] = [`msz ${width} ${height}`, "tna alpha", "tna beta"];

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const counts = RESOURCE_NAMES.map((_, i) =>
        // Sparse, deterministic scatter: a resource shows up when the tile
        // index lines up with the slot, so every stone type appears somewhere.
        (x * 7 + y * 13 + i * 5) % 17 === 0 ? 1 : 0,
      );
      if (counts.some((n) => n > 0)) {
        lines.push(`bct ${x} ${y} ${counts.join(" ")}`);
      }
    }
  }

  return lines;
}

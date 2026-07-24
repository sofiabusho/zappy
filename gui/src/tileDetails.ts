/**
 * Tile detail formatting for the square overlay (G03).
 *
 * Produces structured rows so auditors can distinguish every stone count on a
 * square (AQ16 / AQ17). Pure data — no DOM — so unit tests stay headless.
 */

import { STONE_NAMES } from "./icons.js";
import {
  RESOURCE_NAMES,
  type ResourceName,
  type TileContent,
} from "./protocol.js";

export interface TileDetailRow {
  name: ResourceName;
  count: number;
  /** True for the six stone types (not food). */
  isStone: boolean;
}

export interface TileDetails {
  x: number;
  y: number;
  rows: TileDetailRow[];
  /** Player ids currently standing on this tile (informational; G04 owns chars). */
  playerIds: number[];
}

/** Build detail rows for every resource slot, including zeros (AQ17). */
export function buildTileDetails(
  x: number,
  y: number,
  content: TileContent,
  playerIds: number[] = [],
): TileDetails {
  const stoneSet = new Set<string>(STONE_NAMES);
  const rows: TileDetailRow[] = RESOURCE_NAMES.map((name) => ({
    name,
    count: content[name],
    isStone: stoneSet.has(name),
  }));
  return { x, y, rows, playerIds: [...playerIds] };
}

/** Plain-text summary used by tests and as a11y fallback. */
export function formatTileDetailsText(details: TileDetails): string {
  const lines = [`Square (${details.x}, ${details.y})`];
  for (const row of details.rows) {
    lines.push(`${row.name}: ${row.count}`);
  }
  if (details.playerIds.length > 0) {
    lines.push(
      `players: ${details.playerIds.map((id) => `#${id}`).join(", ")}`,
    );
  } else {
    lines.push("players: (none)");
  }
  return lines.join("\n");
}

/** True when every stone type has an explicit numeric row (AQ17). */
export function stoneCountsDistinguishable(details: TileDetails): boolean {
  const stoneRows = details.rows.filter((r) => r.isStone);
  if (stoneRows.length !== STONE_NAMES.length) {
    return false;
  }
  return stoneRows.every(
    (r) => typeof r.count === "number" && Number.isFinite(r.count),
  );
}

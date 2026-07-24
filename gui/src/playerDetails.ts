/**
 * Player characteristic formatting for the click overlay (G04 / AQ19).
 */

import {
  ORIENTATION_LABELS,
  RESOURCE_NAMES,
  type GuiPlayer,
} from "./protocol.js";

export interface PlayerDetails {
  id: number;
  team: string;
  level: number;
  x: number;
  y: number;
  orientation: string;
  inventory: Array<{ name: string; count: number }>;
}

/** Build a characteristics snapshot from a tracked player. */
export function buildPlayerDetails(player: GuiPlayer): PlayerDetails {
  return {
    id: player.id,
    team: player.team,
    level: player.level,
    x: player.x,
    y: player.y,
    orientation: ORIENTATION_LABELS[player.orientation],
    inventory: RESOURCE_NAMES.map((name) => ({
      name,
      count: player.inventory[name],
    })),
  };
}

/** Plain-text form used by tests and a11y. */
export function formatPlayerDetailsText(details: PlayerDetails): string {
  const lines = [
    `Player #${details.id}`,
    `team: ${details.team}`,
    `level: ${details.level}`,
    `position: (${details.x}, ${details.y})`,
    `orientation: ${details.orientation}`,
    "inventory:",
  ];
  for (const row of details.inventory) {
    lines.push(`  ${row.name}: ${row.count}`);
  }
  return lines.join("\n");
}

/** True when the snapshot exposes the core AQ19 characteristics. */
export function hasCoreCharacteristics(details: PlayerDetails): boolean {
  return (
    Number.isFinite(details.id) &&
    details.team.length > 0 &&
    details.level >= 1 &&
    details.orientation.length > 0 &&
    details.inventory.length === RESOURCE_NAMES.length
  );
}

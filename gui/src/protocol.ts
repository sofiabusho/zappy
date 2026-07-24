/**
 * Zappy server → GUI protocol (G01).
 *
 * The raw subject does not fix a GUI wire format, so this file defines the
 * documented server↔GUI side-channel (see `docs/SDS.md` §12). It is a
 * line-based, `\n`-terminated ASCII protocol that never touches the AI player
 * protocol. G01 consumes the map subset (`msz`, `bct`) plus team names
 * (`tna`); richer entity events (players, broadcasts) are reserved for
 * G02–G05 and parse to `unknown` here so older/newer streams stay compatible.
 *
 * Resource ordering matches the inventory contract used everywhere else in the
 * project (`docs/SDS.md` §5): food, jade, peridot, amber, amethyst, garnet,
 * ammolite.
 */

/** Resource slots on a tile, in the canonical order used by `bct`. */
export const RESOURCE_NAMES = [
  "food",
  "jade",
  "peridot",
  "amber",
  "amethyst",
  "garnet",
  "ammolite",
] as const;

export type ResourceName = (typeof RESOURCE_NAMES)[number];

/** Count of each resource on a single tile. */
export type TileContent = Record<ResourceName, number>;

/** A parsed server → GUI message. */
export type GuiMessage =
  | { kind: "map-size"; width: number; height: number }
  | { kind: "tile"; x: number; y: number; content: TileContent }
  | { kind: "team-name"; name: string }
  | { kind: "unknown"; raw: string };

/** All-zero tile content. */
export function emptyTile(): TileContent {
  return {
    food: 0,
    jade: 0,
    peridot: 0,
    amber: 0,
    amethyst: 0,
    garnet: 0,
    ammolite: 0,
  };
}

/** True when the tile carries at least one resource. */
export function tileHasResources(content: TileContent): boolean {
  return RESOURCE_NAMES.some((name) => content[name] > 0);
}

/** Parse a non-negative integer token; `null` if it is not one. */
function parseCount(token: string | undefined): number | null {
  if (token === undefined || !/^\d+$/.test(token)) {
    return null;
  }
  return Number.parseInt(token, 10);
}

/**
 * Parse one server → GUI line into a {@link GuiMessage}. Unrecognised or
 * malformed lines return `{ kind: "unknown" }` rather than throwing so a live
 * stream can never crash the renderer.
 */
export function parseGuiLine(line: string): GuiMessage {
  const trimmed = line.trim();
  if (trimmed.length === 0) {
    return { kind: "unknown", raw: line };
  }

  const parts = trimmed.split(/\s+/);
  const verb = parts[0];

  switch (verb) {
    case "msz": {
      const width = parseCount(parts[1]);
      const height = parseCount(parts[2]);
      if (width === null || height === null || width === 0 || height === 0) {
        return { kind: "unknown", raw: line };
      }
      return { kind: "map-size", width, height };
    }

    case "bct": {
      const x = parseCount(parts[1]);
      const y = parseCount(parts[2]);
      if (x === null || y === null) {
        return { kind: "unknown", raw: line };
      }
      const content = emptyTile();
      for (let i = 0; i < RESOURCE_NAMES.length; i += 1) {
        const count = parseCount(parts[3 + i]);
        if (count === null) {
          return { kind: "unknown", raw: line };
        }
        content[RESOURCE_NAMES[i]] = count;
      }
      return { kind: "tile", x, y, content };
    }

    case "tna": {
      const name = parts[1];
      if (name === undefined) {
        return { kind: "unknown", raw: line };
      }
      return { kind: "team-name", name };
    }

    default:
      return { kind: "unknown", raw: line };
  }
}

/**
 * Split a raw byte-stream chunk into complete lines, returning the leftover
 * partial line for the next chunk. GUI transports call this so a resource tile
 * that arrives split across two TCP/WebSocket frames is still parsed once.
 */
export function splitLines(
  buffer: string,
  chunk: string,
): { lines: string[]; rest: string } {
  const combined = buffer + chunk;
  const segments = combined.split("\n");
  const rest = segments.pop() ?? "";
  return { lines: segments, rest };
}

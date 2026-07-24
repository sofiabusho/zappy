/**
 * Zappy server → GUI protocol (G01 + G02).
 *
 * The raw subject does not fix a GUI wire format, so this file defines the
 * documented server↔GUI side-channel (see `docs/SDS.md` §12). It is a
 * line-based, `\n`-terminated ASCII protocol that never touches the AI player
 * protocol. Map subset: `msz`, `bct`, `tna`. Player subset (G02/G04): `pnw`,
 * `ppo`, `plv`, `pin`, `pdi`. Sound subset (G05): `pbc`, `pic`. Unknown verbs
 * stay `{ kind: "unknown" }`.
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

/** Cardinal facing used by player spawn/position events (1=N 2=E 3=S 4=W). */
export type Orientation = 1 | 2 | 3 | 4;

export const ORIENTATION_LABELS: Record<Orientation, string> = {
  1: "N",
  2: "E",
  3: "S",
  4: "W",
};

/** Starting inventory when a player appears before any `pin` (AQ21 defaults). */
export function defaultPlayerInventory(): TileContent {
  return {
    food: 10,
    jade: 0,
    peridot: 0,
    amber: 0,
    amethyst: 0,
    garnet: 0,
    ammolite: 0,
  };
}

/** A player entity tracked for map icons + characteristics (G02 / G04). */
export interface GuiPlayer {
  id: number;
  x: number;
  y: number;
  orientation: Orientation;
  level: number;
  team: string;
  /** Inventory counts (food + six stones); updated by `pin`. */
  inventory: TileContent;
}

/** A parsed server → GUI message. */
export type GuiMessage =
  | { kind: "map-size"; width: number; height: number }
  | { kind: "tile"; x: number; y: number; content: TileContent }
  | { kind: "team-name"; name: string }
  | {
      kind: "player-new";
      id: number;
      x: number;
      y: number;
      orientation: Orientation;
      level: number;
      team: string;
    }
  | {
      kind: "player-pos";
      id: number;
      x: number;
      y: number;
      orientation: Orientation;
    }
  | { kind: "player-level"; id: number; level: number }
  | {
      kind: "player-inventory";
      id: number;
      x: number;
      y: number;
      inventory: TileContent;
    }
  | { kind: "player-dead"; id: number }
  | { kind: "broadcast"; fromId: number; text: string }
  | { kind: "sound"; toId: number; k: number; text: string }
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

/** Resource kinds present on a tile (count > 0), in canonical order. */
export function presentResources(content: TileContent): ResourceName[] {
  return RESOURCE_NAMES.filter((name) => content[name] > 0);
}

/** Parse a non-negative integer token; `null` if it is not one. */
function parseCount(token: string | undefined): number | null {
  if (token === undefined || !/^\d+$/.test(token)) {
    return null;
  }
  return Number.parseInt(token, 10);
}

/** Parse `#N` player id token. */
function parsePlayerId(token: string | undefined): number | null {
  if (token === undefined || !/^#\d+$/.test(token)) {
    return null;
  }
  return Number.parseInt(token.slice(1), 10);
}

function parseOrientation(token: string | undefined): Orientation | null {
  const n = parseCount(token);
  if (n === null || n < 1 || n > 4) {
    return null;
  }
  return n as Orientation;
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

    case "pnw": {
      // pnw #<id> <x> <y> <o> <level> <team>
      const id = parsePlayerId(parts[1]);
      const x = parseCount(parts[2]);
      const y = parseCount(parts[3]);
      const orientation = parseOrientation(parts[4]);
      const level = parseCount(parts[5]);
      const team = parts[6];
      if (
        id === null ||
        x === null ||
        y === null ||
        orientation === null ||
        level === null ||
        team === undefined
      ) {
        return { kind: "unknown", raw: line };
      }
      return { kind: "player-new", id, x, y, orientation, level, team };
    }

    case "ppo": {
      // ppo #<id> <x> <y> <o>
      const id = parsePlayerId(parts[1]);
      const x = parseCount(parts[2]);
      const y = parseCount(parts[3]);
      const orientation = parseOrientation(parts[4]);
      if (id === null || x === null || y === null || orientation === null) {
        return { kind: "unknown", raw: line };
      }
      return { kind: "player-pos", id, x, y, orientation };
    }

    case "plv": {
      // plv #<id> <level>
      const id = parsePlayerId(parts[1]);
      const level = parseCount(parts[2]);
      if (id === null || level === null) {
        return { kind: "unknown", raw: line };
      }
      return { kind: "player-level", id, level };
    }

    case "pin": {
      // pin #<id> <x> <y> <food> <jade> <peridot> <amber> <amethyst> <garnet> <ammolite>
      const id = parsePlayerId(parts[1]);
      const x = parseCount(parts[2]);
      const y = parseCount(parts[3]);
      if (id === null || x === null || y === null) {
        return { kind: "unknown", raw: line };
      }
      const inventory = emptyTile();
      for (let i = 0; i < RESOURCE_NAMES.length; i += 1) {
        const count = parseCount(parts[4 + i]);
        if (count === null) {
          return { kind: "unknown", raw: line };
        }
        inventory[RESOURCE_NAMES[i]] = count;
      }
      return { kind: "player-inventory", id, x, y, inventory };
    }

    case "pdi": {
      const id = parsePlayerId(parts[1]);
      if (id === null) {
        return { kind: "unknown", raw: line };
      }
      return { kind: "player-dead", id };
    }

    case "pbc": {
      // pbc #<fromId> <text…>  — broadcast emitted by a player (GUI ripple)
      const fromId = parsePlayerId(parts[1]);
      if (fromId === null || parts.length < 3) {
        return { kind: "unknown", raw: line };
      }
      const text = parts.slice(2).join(" ");
      return { kind: "broadcast", fromId, text };
    }

    case "pic": {
      // pic #<toId> <K> <text…>  — listener hears sound from sector K
      const toId = parsePlayerId(parts[1]);
      const k = parseCount(parts[2]);
      if (toId === null || k === null || k > 8 || parts.length < 4) {
        return { kind: "unknown", raw: line };
      }
      const text = parts.slice(3).join(" ");
      return { kind: "sound", toId, k, text };
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

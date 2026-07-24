/**
 * Live world model fed by the server → GUI protocol (G01 / G02).
 *
 * Holds map dimensions, per-tile resource counts, known teams, and player
 * entities for icon rendering. The grid is toroidal (RQ03 / AQ10): coordinate
 * lookups wrap in both axes so the renderer and later click-hit-testing never
 * index out of range.
 */

import {
  type GuiMessage,
  type GuiPlayer,
  type TileContent,
  emptyTile,
} from "./protocol.js";

export class WorldState {
  private width = 0;
  private height = 0;
  private tiles: TileContent[] = [];
  private readonly teams = new Set<string>();
  private readonly players = new Map<number, GuiPlayer>();

  /** Map width in tiles (0 until `msz` arrives). */
  get mapWidth(): number {
    return this.width;
  }

  /** Map height in tiles (0 until `msz` arrives). */
  get mapHeight(): number {
    return this.height;
  }

  /** True once the map size is known and a grid can be drawn. */
  isReady(): boolean {
    return this.width > 0 && this.height > 0;
  }

  /** Known team names, in insertion order. */
  teamNames(): string[] {
    return [...this.teams];
  }

  /** Living players currently tracked for icons. */
  allPlayers(): GuiPlayer[] {
    return [...this.players.values()];
  }

  /** Players currently on tile `(x, y)` (toroidal). */
  playersAt(x: number, y: number): GuiPlayer[] {
    if (!this.isReady()) {
      return [];
    }
    const wx = WorldState.wrap(x, this.width);
    const wy = WorldState.wrap(y, this.height);
    return this.allPlayers().filter((p) => p.x === wx && p.y === wy);
  }

  /** Wrap a coordinate onto the torus; returns a value in `[0, size)`. */
  private static wrap(value: number, size: number): number {
    if (size <= 0) {
      return 0;
    }
    return ((value % size) + size) % size;
  }

  /**
   * Tile content at `(x, y)` with toroidal wrapping, or `null` if the map size
   * is not known yet.
   */
  tileAt(x: number, y: number): TileContent | null {
    if (!this.isReady()) {
      return null;
    }
    const wx = WorldState.wrap(x, this.width);
    const wy = WorldState.wrap(y, this.height);
    return this.tiles[wy * this.width + wx] ?? emptyTile();
  }

  private resize(width: number, height: number): void {
    this.width = width;
    this.height = height;
    this.tiles = Array.from({ length: width * height }, emptyTile);
    this.players.clear();
  }

  private upsertPlayer(player: GuiPlayer): void {
    if (!this.isReady()) {
      return;
    }
    this.players.set(player.id, {
      ...player,
      x: WorldState.wrap(player.x, this.width),
      y: WorldState.wrap(player.y, this.height),
    });
  }

  /** Fold one parsed message into the world state. */
  apply(message: GuiMessage): void {
    switch (message.kind) {
      case "map-size":
        this.resize(message.width, message.height);
        break;

      case "tile": {
        if (!this.isReady()) {
          break;
        }
        const wx = WorldState.wrap(message.x, this.width);
        const wy = WorldState.wrap(message.y, this.height);
        this.tiles[wy * this.width + wx] = message.content;
        break;
      }

      case "team-name":
        this.teams.add(message.name);
        break;

      case "player-new":
        this.teams.add(message.team);
        this.upsertPlayer({
          id: message.id,
          x: message.x,
          y: message.y,
          orientation: message.orientation,
          level: message.level,
          team: message.team,
        });
        break;

      case "player-pos": {
        const existing = this.players.get(message.id);
        if (existing === undefined) {
          // Position update before spawn — track with placeholder metadata.
          this.upsertPlayer({
            id: message.id,
            x: message.x,
            y: message.y,
            orientation: message.orientation,
            level: 1,
            team: "?",
          });
          break;
        }
        this.upsertPlayer({
          ...existing,
          x: message.x,
          y: message.y,
          orientation: message.orientation,
        });
        break;
      }

      case "player-dead":
        this.players.delete(message.id);
        break;

      case "unknown":
        break;
    }
  }
}

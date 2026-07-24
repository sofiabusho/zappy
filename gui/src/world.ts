/**
 * Live world model fed by the server → GUI protocol (G01).
 *
 * Holds map dimensions, per-tile resource counts, and the set of known team
 * names. The grid is toroidal (RQ03 / AQ10): coordinate lookups wrap in both
 * axes so the renderer and later click-hit-testing never index out of range.
 */

import {
  type GuiMessage,
  type TileContent,
  emptyTile,
} from "./protocol.js";

export class WorldState {
  private width = 0;
  private height = 0;
  private tiles: TileContent[] = [];
  private readonly teams = new Set<string>();

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

      case "unknown":
        break;
    }
  }
}

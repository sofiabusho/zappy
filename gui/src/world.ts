/**
 * Live world model fed by the server → GUI protocol (G01–G05).
 *
 * Holds map, players, and recent sound/broadcast events for visualization.
 */

import {
  type GuiMessage,
  type GuiPlayer,
  type TileContent,
  defaultPlayerInventory,
  emptyTile,
} from "./protocol.js";
import {
  SOUND_TTL_MS,
  isValidSoundK,
  type SoundEvent,
  type SoundK,
} from "./sound.js";

export class WorldState {
  private width = 0;
  private height = 0;
  private tiles: TileContent[] = [];
  private readonly teams = new Set<string>();
  private readonly players = new Map<number, GuiPlayer>();
  private sounds: SoundEvent[] = [];
  private nextSoundId = 1;
  private nowFn: () => number = () =>
    typeof performance !== "undefined" ? performance.now() : Date.now();

  /** Inject a clock (tests). */
  setNow(fn: () => number): void {
    this.nowFn = fn;
  }

  get mapWidth(): number {
    return this.width;
  }

  get mapHeight(): number {
    return this.height;
  }

  isReady(): boolean {
    return this.width > 0 && this.height > 0;
  }

  teamNames(): string[] {
    return [...this.teams];
  }

  allPlayers(): GuiPlayer[] {
    return [...this.players.values()];
  }

  playerById(id: number): GuiPlayer | null {
    return this.players.get(id) ?? null;
  }

  playersAt(x: number, y: number): GuiPlayer[] {
    if (!this.isReady()) {
      return [];
    }
    const wx = WorldState.wrap(x, this.width);
    const wy = WorldState.wrap(y, this.height);
    return this.allPlayers().filter((p) => p.x === wx && p.y === wy);
  }

  /** Active (non-expired) sound / broadcast events for drawing + feed. */
  activeSounds(now = this.nowFn()): SoundEvent[] {
    this.pruneSounds(now);
    return [...this.sounds];
  }

  private static wrap(value: number, size: number): number {
    if (size <= 0) {
      return 0;
    }
    return ((value % size) + size) % size;
  }

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
    this.sounds = [];
  }

  private upsertPlayer(player: GuiPlayer): void {
    if (!this.isReady()) {
      return;
    }
    this.players.set(player.id, {
      ...player,
      inventory: { ...player.inventory },
      x: WorldState.wrap(player.x, this.width),
      y: WorldState.wrap(player.y, this.height),
    });
  }

  private pruneSounds(now: number): void {
    this.sounds = this.sounds.filter((s) => now - s.bornAt < SOUND_TTL_MS);
  }

  private pushSound(partial: Omit<SoundEvent, "id" | "bornAt">): void {
    const now = this.nowFn();
    this.pruneSounds(now);
    this.sounds.push({
      ...partial,
      id: this.nextSoundId,
      bornAt: now,
    });
    this.nextSoundId += 1;
    // Cap feed length.
    if (this.sounds.length > 24) {
      this.sounds = this.sounds.slice(-24);
    }
  }

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
          inventory: defaultPlayerInventory(),
        });
        break;

      case "player-pos": {
        const existing = this.players.get(message.id);
        if (existing === undefined) {
          this.upsertPlayer({
            id: message.id,
            x: message.x,
            y: message.y,
            orientation: message.orientation,
            level: 1,
            team: "?",
            inventory: defaultPlayerInventory(),
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

      case "player-level": {
        const existing = this.players.get(message.id);
        if (existing === undefined) {
          break;
        }
        this.upsertPlayer({ ...existing, level: message.level });
        break;
      }

      case "player-inventory": {
        const existing = this.players.get(message.id);
        if (existing === undefined) {
          this.upsertPlayer({
            id: message.id,
            x: message.x,
            y: message.y,
            orientation: 1,
            level: 1,
            team: "?",
            inventory: { ...message.inventory },
          });
          break;
        }
        this.upsertPlayer({
          ...existing,
          x: message.x,
          y: message.y,
          inventory: { ...message.inventory },
        });
        break;
      }

      case "player-dead":
        this.players.delete(message.id);
        break;

      case "broadcast": {
        const from = this.players.get(message.fromId);
        this.pushSound({
          kind: "emit",
          fromId: message.fromId,
          toId: null,
          k: null,
          text: message.text,
          x: from?.x ?? 0,
          y: from?.y ?? 0,
        });
        break;
      }

      case "sound": {
        if (!isValidSoundK(message.k)) {
          break;
        }
        const k = message.k as SoundK;
        const to = this.players.get(message.toId);
        this.pushSound({
          kind: "hear",
          fromId: null,
          toId: message.toId,
          k,
          text: message.text,
          x: to?.x ?? 0,
          y: to?.y ?? 0,
        });
        break;
      }

      case "unknown":
        break;
    }
  }
}

/**
 * Sound / broadcast visualization helpers (G05 / AQ20).
 *
 * Direction K matches `docs/SDS.md` §9 (receiver-relative sectors 0..8).
 * No game engine — pure Canvas path drawing + a DOM sound feed.
 */

import { type Orientation } from "./protocol.js";

/** Sound sector K ∈ {0..8}; 0 = same tile as receiver. */
export type SoundK = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;

export interface SoundEvent {
  /** Unique id for list keys / fade. */
  id: number;
  /** Emitter player id when known. */
  fromId: number | null;
  /** Listener player id for directional `pic` events. */
  toId: number | null;
  /** Direction heard by the listener (null for emit-only ripples). */
  k: SoundK | null;
  text: string;
  /** Map tile of the emitter (for ripples). */
  x: number;
  y: number;
  /** performance.now() when the event was recorded. */
  bornAt: number;
  kind: "emit" | "hear";
}

/** How long a sound stays visible (ms). */
export const SOUND_TTL_MS = 3500;

/**
 * Local (dx, dy) for sector K when the receiver faces North (front = −Y).
 * Matches the subject ring:
 * ```
 *   2 1 8
 *   3 P 7
 *   4 5 6
 * ```
 */
export function sectorLocalOffset(k: SoundK): { dx: number; dy: number } {
  switch (k) {
    case 0:
      return { dx: 0, dy: 0 };
    case 1:
      return { dx: 0, dy: -1 };
    case 2:
      return { dx: -1, dy: -1 };
    case 3:
      return { dx: -1, dy: 0 };
    case 4:
      return { dx: -1, dy: 1 };
    case 5:
      return { dx: 0, dy: 1 };
    case 6:
      return { dx: 1, dy: 1 };
    case 7:
      return { dx: 1, dy: 0 };
    case 8:
      return { dx: 1, dy: -1 };
  }
}

/** Normalize signed zero so vector equality stays stable in tests/UI. */
function nz(n: number): number {
  return Object.is(n, -0) ? 0 : n;
}

/** Rotate a facing-local offset into map axes for orientation 1=N…4=W. */
export function rotateByOrientation(
  dx: number,
  dy: number,
  orientation: Orientation,
): { dx: number; dy: number } {
  switch (orientation) {
    case 1: // N — local already map-aligned
      return { dx: nz(dx), dy: nz(dy) };
    case 2: // E — local front → +X
      return { dx: nz(-dy), dy: nz(dx) };
    case 3: // S
      return { dx: nz(-dx), dy: nz(-dy) };
    case 4: // W
      return { dx: nz(dy), dy: nz(-dx) };
  }
}

/** Absolute map unit vector pointing toward the sound source from the listener. */
export function soundArrivalVector(
  k: SoundK,
  orientation: Orientation,
): { dx: number; dy: number } {
  const local = sectorLocalOffset(k);
  return rotateByOrientation(local.dx, local.dy, orientation);
}

export function isValidSoundK(n: number): n is SoundK {
  return Number.isInteger(n) && n >= 0 && n <= 8;
}

/** Fade factor 1→0 over SOUND_TTL_MS. */
export function soundAlpha(bornAt: number, now: number): number {
  const age = now - bornAt;
  if (age <= 0) {
    return 1;
  }
  if (age >= SOUND_TTL_MS) {
    return 0;
  }
  return 1 - age / SOUND_TTL_MS;
}

/** Ripple radius in cell units over the event lifetime. */
export function soundRippleRadius(bornAt: number, now: number, cell: number): number {
  const t = Math.min(1, Math.max(0, (now - bornAt) / SOUND_TTL_MS));
  return cell * (0.35 + t * 1.8);
}

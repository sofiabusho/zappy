/**
 * G05 unit tests — broadcast / sound visualization (RQ21, AQ20).
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { parseGuiLine } from "./protocol.js";
import {
  SOUND_TTL_MS,
  isValidSoundK,
  rotateByOrientation,
  sectorLocalOffset,
  soundAlpha,
  soundArrivalVector,
} from "./sound.js";
import { WorldState } from "./world.js";
import { demoStream } from "./demo.js";

test("parseGuiLine understands pbc and pic", () => {
  assert.deepEqual(parseGuiLine("pbc #1 hello world"), {
    kind: "broadcast",
    fromId: 1,
    text: "hello world",
  });
  assert.deepEqual(parseGuiLine("pic #2 3 meetup"), {
    kind: "sound",
    toId: 2,
    k: 3,
    text: "meetup",
  });
  assert.equal(parseGuiLine("pic #2 9 bad").kind, "unknown");
  assert.equal(parseGuiLine("pbc #1").kind, "unknown");
});

test("isValidSoundK accepts 0..8 only", () => {
  for (let k = 0; k <= 8; k += 1) {
    assert.equal(isValidSoundK(k), true);
  }
  assert.equal(isValidSoundK(-1), false);
  assert.equal(isValidSoundK(9), false);
});

test("sectorLocalOffset matches SDS ring (facing North)", () => {
  assert.deepEqual(sectorLocalOffset(0), { dx: 0, dy: 0 });
  assert.deepEqual(sectorLocalOffset(1), { dx: 0, dy: -1 });
  assert.deepEqual(sectorLocalOffset(3), { dx: -1, dy: 0 });
  assert.deepEqual(sectorLocalOffset(5), { dx: 0, dy: 1 });
  assert.deepEqual(sectorLocalOffset(7), { dx: 1, dy: 0 });
});

test("rotateByOrientation maps local front to facing", () => {
  // Local front (−Y) when facing East → +X.
  assert.deepEqual(rotateByOrientation(0, -1, 2), { dx: 1, dy: 0 });
  assert.deepEqual(rotateByOrientation(0, -1, 3), { dx: 0, dy: 1 });
  assert.deepEqual(rotateByOrientation(0, -1, 4), { dx: -1, dy: 0 });
});

test("soundArrivalVector K=1 points toward facing", () => {
  assert.deepEqual(soundArrivalVector(1, 1), { dx: 0, dy: -1 }); // N
  assert.deepEqual(soundArrivalVector(1, 2), { dx: 1, dy: 0 }); // E
});

test("soundAlpha fades over SOUND_TTL_MS", () => {
  assert.equal(soundAlpha(100, 100), 1);
  assert.equal(soundAlpha(100, 100 + SOUND_TTL_MS), 0);
  assert.ok(soundAlpha(100, 100 + SOUND_TTL_MS / 2) > 0.4);
  assert.ok(soundAlpha(100, 100 + SOUND_TTL_MS / 2) < 0.6);
});

test("world records emit ripples and hear events (AQ20)", () => {
  let now = 1000;
  const world = new WorldState();
  world.setNow(() => now);
  world.apply(parseGuiLine("msz 10 10"));
  world.apply(parseGuiLine("pnw #1 2 2 1 1 alpha"));
  world.apply(parseGuiLine("pnw #2 4 2 1 1 beta"));
  world.apply(parseGuiLine("pbc #1 rally"));
  world.apply(parseGuiLine("pic #2 3 rally"));

  const sounds = world.activeSounds(now);
  assert.equal(sounds.length, 2);
  const emit = sounds.find((s) => s.kind === "emit");
  const hear = sounds.find((s) => s.kind === "hear");
  assert.ok(emit);
  assert.ok(hear);
  assert.equal(emit.fromId, 1);
  assert.equal(emit.x, 2);
  assert.equal(emit.y, 2);
  assert.equal(emit.text, "rally");
  assert.equal(hear.toId, 2);
  assert.equal(hear.k, 3);
  assert.equal(hear.text, "rally");

  now += SOUND_TTL_MS + 1;
  assert.equal(world.activeSounds(now).length, 0);
});

test("demo stream includes pbc and pic for offline sound viz", () => {
  const lines = demoStream();
  assert.ok(lines.some((l) => l.startsWith("pbc ")));
  assert.ok(lines.some((l) => l.startsWith("pic ")));
});

/**
 * G02 unit tests — icon catalog, protocol player events, demo coverage.
 * Run via `npm test` (tsc build + node --test).
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ICON_SPECS,
  STONE_NAMES,
  iconCatalogComplete,
  iconSlots,
} from "./icons.js";
import {
  RESOURCE_NAMES,
  parseGuiLine,
  presentResources,
} from "./protocol.js";
import { demoCoversIcons, demoStream } from "./demo.js";
import { WorldState } from "./world.js";

test("icon catalog covers food, six stones, and player (AQ18/AQ28)", () => {
  assert.equal(STONE_NAMES.length, 6);
  assert.deepEqual([...STONE_NAMES], [
    "jade",
    "peridot",
    "amber",
    "amethyst",
    "garnet",
    "ammolite",
  ]);
  assert.ok(iconCatalogComplete());
  assert.ok(ICON_SPECS.food);
  assert.ok(ICON_SPECS.player);
  for (const stone of STONE_NAMES) {
    assert.ok(ICON_SPECS[stone], `missing icon for ${stone}`);
  }
});

test("icon slots lay out N centers inside a cell", () => {
  const slots = iconSlots(3, 40);
  assert.equal(slots.length, 3);
  for (const s of slots) {
    assert.ok(s.x > 0 && s.x < 40);
    assert.ok(s.y > 0 && s.y < 40);
  }
});

test("parseGuiLine understands pnw / ppo / pdi", () => {
  const born = parseGuiLine("pnw #7 3 4 2 1 alpha");
  assert.deepEqual(born, {
    kind: "player-new",
    id: 7,
    x: 3,
    y: 4,
    orientation: 2,
    level: 1,
    team: "alpha",
  });

  const moved = parseGuiLine("ppo #7 5 4 3");
  assert.deepEqual(moved, {
    kind: "player-pos",
    id: 7,
    x: 5,
    y: 4,
    orientation: 3,
  });

  const dead = parseGuiLine("pdi #7");
  assert.deepEqual(dead, { kind: "player-dead", id: 7 });
});

test("presentResources lists only positive counts in canonical order", () => {
  const names = presentResources({
    food: 1,
    jade: 0,
    peridot: 1,
    amber: 0,
    amethyst: 0,
    garnet: 1,
    ammolite: 0,
  });
  assert.deepEqual(names, ["food", "peridot", "garnet"]);
  assert.deepEqual(RESOURCE_NAMES[0], "food");
});

test("world tracks players for icon rendering", () => {
  const world = new WorldState();
  world.apply({ kind: "map-size", width: 10, height: 10 });
  world.apply({
    kind: "player-new",
    id: 1,
    x: 2,
    y: 3,
    orientation: 1,
    level: 1,
    team: "alpha",
  });
  assert.equal(world.allPlayers().length, 1);
  world.apply({
    kind: "player-pos",
    id: 1,
    x: 9,
    y: 3,
    orientation: 2,
  });
  assert.equal(world.allPlayers()[0]?.x, 9);
  world.apply({ kind: "player-dead", id: 1 });
  assert.equal(world.allPlayers().length, 0);
});

test("demo stream shows food, all stones, and players (AQ18/AQ28)", () => {
  const lines = demoStream(12, 12);
  assert.ok(demoCoversIcons(lines));
  assert.ok(lines.some((l) => l.startsWith("pnw ")));
});

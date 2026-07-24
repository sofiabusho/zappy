/**
 * G04 unit tests — player characteristics overlay data + hit-testing (AQ19).
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  defaultPlayerInventory,
  parseGuiLine,
  type GuiPlayer,
} from "./protocol.js";
import {
  buildPlayerDetails,
  formatPlayerDetailsText,
  hasCoreCharacteristics,
} from "./playerDetails.js";
import {
  computeGridLayout,
  hitTestPlayer,
  type GridLayout,
} from "./layout.js";
import { WorldState } from "./world.js";

function samplePlayer(): GuiPlayer {
  return {
    id: 3,
    x: 4,
    y: 5,
    orientation: 2,
    level: 3,
    team: "alpha",
    inventory: {
      food: 10,
      jade: 1,
      peridot: 0,
      amber: 0,
      amethyst: 0,
      garnet: 2,
      ammolite: 0,
    },
  };
}

test("buildPlayerDetails exposes core characteristics (AQ19)", () => {
  const details = buildPlayerDetails(samplePlayer());
  assert.ok(hasCoreCharacteristics(details));
  assert.equal(details.id, 3);
  assert.equal(details.team, "alpha");
  assert.equal(details.level, 3);
  assert.equal(details.orientation, "E");
  assert.equal(details.inventory.find((r) => r.name === "food")?.count, 10);
  assert.equal(details.inventory.find((r) => r.name === "garnet")?.count, 2);
});

test("formatPlayerDetailsText lists identity and inventory", () => {
  const text = formatPlayerDetailsText(buildPlayerDetails(samplePlayer()));
  assert.match(text, /Player #3/);
  assert.match(text, /team: alpha/);
  assert.match(text, /level: 3/);
  assert.match(text, /orientation: E/);
  assert.match(text, /food: 10/);
  assert.match(text, /garnet: 2/);
});

test("parseGuiLine understands plv and pin", () => {
  assert.deepEqual(parseGuiLine("plv #3 4"), {
    kind: "player-level",
    id: 3,
    level: 4,
  });
  const pin = parseGuiLine("pin #3 1 2 9 1 0 0 0 0 0");
  assert.equal(pin.kind, "player-inventory");
  if (pin.kind === "player-inventory") {
    assert.equal(pin.inventory.food, 9);
    assert.equal(pin.inventory.jade, 1);
  }
});

test("world applies pin/plv into player characteristics", () => {
  const world = new WorldState();
  world.apply({ kind: "map-size", width: 10, height: 10 });
  world.apply({
    kind: "player-new",
    id: 1,
    x: 2,
    y: 2,
    orientation: 1,
    level: 1,
    team: "beta",
  });
  const born = world.playerById(1);
  assert.ok(born);
  assert.deepEqual(born!.inventory, defaultPlayerInventory());

  world.apply({ kind: "player-level", id: 1, level: 5 });
  world.apply({
    kind: "player-inventory",
    id: 1,
    x: 3,
    y: 2,
    inventory: {
      food: 4,
      jade: 0,
      peridot: 1,
      amber: 0,
      amethyst: 0,
      garnet: 0,
      ammolite: 0,
    },
  });
  const updated = world.playerById(1)!;
  assert.equal(updated.level, 5);
  assert.equal(updated.x, 3);
  assert.equal(updated.inventory.food, 4);
  assert.equal(updated.inventory.peridot, 1);
});

test("hitTestPlayer finds the player under the cursor", () => {
  const world = new WorldState();
  world.apply({ kind: "map-size", width: 10, height: 10 });
  world.apply({
    kind: "player-new",
    id: 7,
    x: 1,
    y: 1,
    orientation: 1,
    level: 1,
    team: "alpha",
  });
  const layout: GridLayout = {
    cols: 10,
    rows: 10,
    cell: 20,
    offsetX: 0,
    offsetY: 0,
    gridW: 200,
    gridH: 200,
    mapHeight: 200,
  };
  // Center of tile (1,1) at cell 20 → (30, 30)
  assert.equal(hitTestPlayer(30, 30, layout, world), 7);
  assert.equal(hitTestPlayer(0, 0, layout, world), null);

  const computed = computeGridLayout(400, 400, world);
  assert.ok(computed);
});

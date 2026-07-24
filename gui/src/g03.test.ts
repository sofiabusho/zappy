/**
 * G03 unit tests — tile detail counts + grid hit-testing (AQ16 / AQ17).
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { STONE_NAMES } from "./icons.js";
import { emptyTile, type TileContent } from "./protocol.js";
import {
  buildTileDetails,
  formatTileDetailsText,
  stoneCountsDistinguishable,
} from "./tileDetails.js";
import { computeGridLayout, hitTestTile, type GridLayout } from "./layout.js";
import { WorldState } from "./world.js";

function sampleTile(): TileContent {
  return {
    food: 1,
    jade: 0,
    peridot: 1,
    amber: 0,
    amethyst: 2,
    garnet: 0,
    ammolite: 1,
  };
}

test("buildTileDetails lists every resource with a numeric count (AQ17)", () => {
  const details = buildTileDetails(3, 4, sampleTile(), [1, 9]);
  assert.equal(details.x, 3);
  assert.equal(details.y, 4);
  assert.equal(details.rows.length, 7);
  assert.ok(stoneCountsDistinguishable(details));
  const stones = details.rows.filter((r) => r.isStone);
  assert.equal(stones.length, STONE_NAMES.length);
  assert.equal(
    details.rows.find((r) => r.name === "amethyst")?.count,
    2,
  );
  assert.equal(details.rows.find((r) => r.name === "jade")?.count, 0);
  assert.deepEqual(details.playerIds, [1, 9]);
});

test("formatTileDetailsText includes each stone count explicitly", () => {
  const text = formatTileDetailsText(
    buildTileDetails(0, 0, sampleTile()),
  );
  for (const stone of STONE_NAMES) {
    assert.match(text, new RegExp(`${stone}: \\d+`));
  }
  assert.match(text, /food: 1/);
  assert.match(text, /Square \(0, 0\)/);
});

test("empty tile still exposes zero counts for all stones", () => {
  const details = buildTileDetails(1, 1, emptyTile());
  assert.ok(stoneCountsDistinguishable(details));
  for (const row of details.rows) {
    assert.equal(row.count, 0);
  }
});

test("hitTestTile maps canvas pixels to grid coordinates", () => {
  const layout: GridLayout = {
    cols: 10,
    rows: 8,
    cell: 20,
    offsetX: 50,
    offsetY: 30,
    gridW: 200,
    gridH: 160,
    mapHeight: 200,
  };
  assert.deepEqual(hitTestTile(50, 30, layout), { x: 0, y: 0 });
  assert.deepEqual(hitTestTile(69, 49, layout), { x: 0, y: 0 });
  assert.deepEqual(hitTestTile(70, 50, layout), { x: 1, y: 1 });
  assert.equal(hitTestTile(10, 30, layout), null);
  assert.equal(hitTestTile(50, 200, layout), null);
});

test("computeGridLayout + world.playersAt support square inspect", () => {
  const world = new WorldState();
  world.apply({ kind: "map-size", width: 12, height: 12 });
  world.apply({
    kind: "tile",
    x: 2,
    y: 3,
    content: sampleTile(),
  });
  world.apply({
    kind: "player-new",
    id: 5,
    x: 2,
    y: 3,
    orientation: 1,
    level: 1,
    team: "alpha",
  });

  const layout = computeGridLayout(640, 480, world);
  assert.ok(layout);
  assert.ok(layout!.cell >= 1);

  const players = world.playersAt(2, 3);
  assert.equal(players.length, 1);
  assert.equal(players[0]?.id, 5);

  const details = buildTileDetails(
    2,
    3,
    world.tileAt(2, 3)!,
    players.map((p) => p.id),
  );
  assert.ok(stoneCountsDistinguishable(details));
  assert.equal(details.playerIds[0], 5);
});

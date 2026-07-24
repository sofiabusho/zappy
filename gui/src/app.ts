/**
 * GUI entry point (G01–G04).
 *
 * Click a player icon for characteristics (G04 / AQ19), or a bare square for
 * resource counts (G03). Player hits take priority when the cursor is on an
 * icon.
 */

import { parseGuiLine, emptyTile } from "./protocol.js";
import { WorldState } from "./world.js";
import { CanvasRenderer } from "./renderer.js";
import {
  ReplayTransport,
  WebSocketTransport,
  type Transport,
  type TransportStatus,
} from "./transport.js";
import { demoStream } from "./demo.js";
import {
  computeGridLayout,
  eventToCanvasPoint,
  hitTestPlayer,
  hitTestTile,
} from "./layout.js";
import { buildTileDetails } from "./tileDetails.js";
import { buildPlayerDetails } from "./playerDetails.js";
import { PlayerOverlay, TileOverlay } from "./overlay.js";

function requireElement<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (el === null) {
    throw new Error(`missing #${id} element`);
  }
  return el as T;
}

function chooseTransport(): { transport: Transport; label: string } {
  const params = new URLSearchParams(window.location.search);
  const ws = params.get("ws");
  if (ws !== null && ws.length > 0) {
    return { transport: new WebSocketTransport(ws), label: `bridge ${ws}` };
  }
  return { transport: new ReplayTransport(demoStream()), label: "offline demo" };
}

function start(): void {
  const canvas = requireElement<HTMLCanvasElement>("map");
  const statusEl = requireElement<HTMLElement>("status");
  const dimsEl = requireElement<HTMLElement>("dims");
  const mainEl = requireElement<HTMLElement>("stage");

  const world = new WorldState();
  const renderer = new CanvasRenderer(canvas);
  const tileOverlay = new TileOverlay(mainEl);
  const playerOverlay = new PlayerOverlay(mainEl);

  const resize = (): void => {
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(rect.width * dpr));
    canvas.height = Math.max(1, Math.floor(rect.height * dpr));
    renderer.render(world);
  };

  let frameQueued = false;
  const scheduleRender = (): void => {
    if (frameQueued) {
      return;
    }
    frameQueued = true;
    window.requestAnimationFrame(() => {
      frameQueued = false;
      renderer.render(world);
      dimsEl.textContent = world.isReady()
        ? `${world.mapWidth}×${world.mapHeight}`
        : "—";
    });
  };

  const dismissOverlays = (): void => {
    tileOverlay.hide();
    playerOverlay.hide();
    renderer.setSelectedTile(null);
    renderer.setSelectedPlayer(null);
    scheduleRender();
  };

  const inspectTile = (
    tileX: number,
    tileY: number,
    clientX: number,
    clientY: number,
  ): void => {
    playerOverlay.hide();
    renderer.setSelectedPlayer(null);
    const content = world.tileAt(tileX, tileY) ?? emptyTile();
    const playerIds = world.playersAt(tileX, tileY).map((p) => p.id);
    const details = buildTileDetails(tileX, tileY, content, playerIds);
    tileOverlay.show(details, clientX, clientY);
    renderer.setSelectedTile({ x: tileX, y: tileY });
    scheduleRender();
  };

  const inspectPlayer = (
    playerId: number,
    clientX: number,
    clientY: number,
  ): void => {
    const player = world.playerById(playerId);
    if (player === null) {
      return;
    }
    tileOverlay.hide();
    renderer.setSelectedTile(null);
    const details = buildPlayerDetails(player);
    playerOverlay.show(details, clientX, clientY);
    renderer.setSelectedPlayer(playerId);
    scheduleRender();
  };

  canvas.addEventListener("click", (event: MouseEvent) => {
    if (!world.isReady()) {
      return;
    }
    const layout = computeGridLayout(canvas.width, canvas.height, world);
    if (layout === null) {
      return;
    }
    const point = eventToCanvasPoint(canvas, event.clientX, event.clientY);
    const playerId = hitTestPlayer(point.x, point.y, layout, world);
    if (playerId !== null) {
      inspectPlayer(playerId, event.clientX, event.clientY);
      return;
    }
    const hit = hitTestTile(point.x, point.y, layout);
    if (hit === null) {
      dismissOverlays();
      return;
    }
    inspectTile(hit.x, hit.y, event.clientX, event.clientY);
  });

  window.addEventListener("keydown", (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      dismissOverlays();
    }
  });

  const { transport, label } = chooseTransport();

  transport.onStatus((status: TransportStatus) => {
    statusEl.textContent = `${status} · ${label}`;
    statusEl.dataset.status = status;
  });

  transport.onLine((line: string) => {
    world.apply(parseGuiLine(line));
    scheduleRender();
  });

  window.addEventListener("resize", resize);
  resize();
  transport.connect();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}

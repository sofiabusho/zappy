/**
 * GUI entry point (G01–G03).
 *
 * Wires a {@link Transport} → protocol parse → {@link WorldState} → {@link
 * CanvasRenderer}. Click a square for a floating detail panel with resource
 * counts (G03 / AQ16 / AQ17). Player characteristic overlays are G04.
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
  hitTestTile,
} from "./layout.js";
import { buildTileDetails } from "./tileDetails.js";
import { TileOverlay } from "./overlay.js";

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
  const overlay = new TileOverlay(mainEl);

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

  const dismissOverlay = (): void => {
    overlay.hide();
    renderer.setSelectedTile(null);
    scheduleRender();
  };

  const inspectTile = (
    tileX: number,
    tileY: number,
    clientX: number,
    clientY: number,
  ): void => {
    const content = world.tileAt(tileX, tileY) ?? emptyTile();
    const playerIds = world.playersAt(tileX, tileY).map((p) => p.id);
    const details = buildTileDetails(tileX, tileY, content, playerIds);
    overlay.show(details, clientX, clientY);
    renderer.setSelectedTile({ x: tileX, y: tileY });
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
    const hit = hitTestTile(point.x, point.y, layout);
    if (hit === null) {
      dismissOverlay();
      return;
    }
    inspectTile(hit.x, hit.y, event.clientX, event.clientY);
  });

  window.addEventListener("keydown", (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      dismissOverlay();
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

/**
 * GUI entry point (G01).
 *
 * Wires a {@link Transport} → protocol parse → {@link WorldState} → {@link
 * CanvasRenderer}. With `?ws=ws://host:port` it connects to a WebSocket bridge
 * in front of the Rust server; otherwise it runs the offline demo stream so
 * the map still renders. Icons (G02), click-details (G03/G04), and sound viz
 * (G05) build on this path.
 */

import { parseGuiLine } from "./protocol.js";
import { WorldState } from "./world.js";
import { CanvasRenderer } from "./renderer.js";
import {
  ReplayTransport,
  WebSocketTransport,
  type Transport,
  type TransportStatus,
} from "./transport.js";
import { demoStream } from "./demo.js";

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

  const world = new WorldState();
  const renderer = new CanvasRenderer(canvas);

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

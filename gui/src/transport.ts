/**
 * GUI transports (G01).
 *
 * A browser Canvas cannot open a raw TCP socket, so the GUI reaches the Rust
 * server through a small WebSocket bridge that forwards the documented
 * server → GUI line protocol verbatim (see `gui/README.md`). {@link
 * WebSocketTransport} speaks to that bridge; {@link ReplayTransport} feeds a
 * scripted stream so the map renders (empty → live) without any server, which
 * is what the offline demo uses.
 */

import { splitLines } from "./protocol.js";

export type TransportStatus = "connecting" | "open" | "closed" | "error";

/** Source of raw server → GUI lines. */
export interface Transport {
  /** Register the per-line callback (called once per complete `\n` line). */
  onLine(cb: (line: string) => void): void;
  /** Register the connection-status callback. */
  onStatus(cb: (status: TransportStatus) => void): void;
  /** Begin streaming. */
  connect(): void;
  /** Stop streaming and release resources. */
  close(): void;
}

/** Shared line/status callback plumbing for concrete transports. */
abstract class BaseTransport implements Transport {
  private lineCb: (line: string) => void = () => {};
  private statusCb: (status: TransportStatus) => void = () => {};

  onLine(cb: (line: string) => void): void {
    this.lineCb = cb;
  }

  onStatus(cb: (status: TransportStatus) => void): void {
    this.statusCb = cb;
  }

  protected emitLine(line: string): void {
    this.lineCb(line);
  }

  protected emitStatus(status: TransportStatus): void {
    this.statusCb(status);
  }

  abstract connect(): void;
  abstract close(): void;
}

/** Streams the GUI protocol over a WebSocket bridge to the TCP server. */
export class WebSocketTransport extends BaseTransport {
  private socket: WebSocket | null = null;
  private buffer = "";

  constructor(private readonly url: string) {
    super();
  }

  connect(): void {
    this.emitStatus("connecting");
    const socket = new WebSocket(this.url);
    this.socket = socket;

    socket.addEventListener("open", () => this.emitStatus("open"));
    socket.addEventListener("close", () => this.emitStatus("closed"));
    socket.addEventListener("error", () => this.emitStatus("error"));
    socket.addEventListener("message", (event: MessageEvent) => {
      const data = typeof event.data === "string" ? event.data : "";
      const { lines, rest } = splitLines(this.buffer, data);
      this.buffer = rest;
      for (const line of lines) {
        this.emitLine(line);
      }
    });
  }

  close(): void {
    this.socket?.close();
    this.socket = null;
  }
}

/**
 * Emits a fixed list of protocol lines on a timer. Used by the offline demo to
 * exercise the full parse → world → render path with no server attached.
 */
export class ReplayTransport extends BaseTransport {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private index = 0;

  constructor(
    private readonly lines: string[],
    private readonly intervalMs = 200,
  ) {
    super();
  }

  connect(): void {
    this.emitStatus("connecting");
    this.emitStatus("open");
    this.pump();
  }

  private pump(): void {
    if (this.index >= this.lines.length) {
      return;
    }
    this.emitLine(this.lines[this.index]);
    this.index += 1;
    this.timer = setTimeout(() => this.pump(), this.intervalMs);
  }

  close(): void {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    this.emitStatus("closed");
  }
}

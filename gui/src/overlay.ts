/**
 * Floating overlays (G03 square details + G04 player characteristics).
 */

import { ICON_SPECS } from "./icons.js";
import {
  formatTileDetailsText,
  type TileDetails,
} from "./tileDetails.js";
import {
  formatPlayerDetailsText,
  type PlayerDetails,
} from "./playerDetails.js";
import { type ResourceName } from "./protocol.js";

function positionPanel(
  panel: HTMLElement,
  clientX: number,
  clientY: number,
): void {
  const margin = 12;
  panel.style.left = `${clientX + margin}px`;
  panel.style.top = `${clientY + margin}px`;
  const rect = panel.getBoundingClientRect();
  let left = clientX + margin;
  let top = clientY + margin;
  if (left + rect.width > window.innerWidth - margin) {
    left = Math.max(margin, clientX - rect.width - margin);
  }
  if (top + rect.height > window.innerHeight - margin) {
    top = Math.max(margin, clientY - rect.height - margin);
  }
  panel.style.left = `${left}px`;
  panel.style.top = `${top}px`;
}

export class TileOverlay {
  private readonly root: HTMLElement;
  private readonly titleEl: HTMLElement;
  private readonly bodyEl: HTMLElement;

  constructor(host: HTMLElement) {
    this.root = document.createElement("aside");
    this.root.id = "tile-overlay";
    this.root.className = "float-overlay";
    this.root.hidden = true;
    this.root.setAttribute("role", "dialog");
    this.root.setAttribute("aria-label", "Square details");

    this.titleEl = document.createElement("h2");
    this.bodyEl = document.createElement("div");
    this.bodyEl.className = "overlay-body";

    const hint = document.createElement("p");
    hint.className = "overlay-hint";
    hint.textContent = "Click empty space or press Esc to dismiss";

    this.root.append(this.titleEl, this.bodyEl, hint);
    host.append(this.root);
  }

  /** Show details near `(clientX, clientY)` viewport coordinates. */
  show(details: TileDetails, clientX: number, clientY: number): void {
    this.titleEl.textContent = `Square (${details.x}, ${details.y})`;
    this.bodyEl.replaceChildren();

    const list = document.createElement("ul");
    for (const row of details.rows) {
      const li = document.createElement("li");
      li.dataset.resource = row.name;
      if (row.isStone) {
        li.dataset.stone = "true";
      }
      const swatch = document.createElement("span");
      swatch.className = "swatch";
      swatch.style.background = ICON_SPECS[row.name].color;
      const label = document.createElement("span");
      label.className = "name";
      label.textContent = row.name;
      const count = document.createElement("span");
      count.className = "count";
      count.textContent = String(row.count);
      li.append(swatch, label, count);
      list.append(li);
    }
    this.bodyEl.append(list);

    const players = document.createElement("p");
    players.className = "overlay-meta";
    players.textContent =
      details.playerIds.length > 0
        ? `players: ${details.playerIds.map((id) => `#${id}`).join(", ")}`
        : "players: (none)";
    this.bodyEl.append(players);

    this.root.dataset.text = formatTileDetailsText(details);
    this.root.hidden = false;
    positionPanel(this.root, clientX, clientY);
  }

  hide(): void {
    this.root.hidden = true;
  }

  get visible(): boolean {
    return !this.root.hidden;
  }

  get textSnapshot(): string {
    return this.root.dataset.text ?? "";
  }
}

/** Floating player-characteristics panel (G04 / AQ19). */
export class PlayerOverlay {
  private readonly root: HTMLElement;
  private readonly titleEl: HTMLElement;
  private readonly bodyEl: HTMLElement;

  constructor(host: HTMLElement) {
    this.root = document.createElement("aside");
    this.root.id = "player-overlay";
    this.root.className = "float-overlay";
    this.root.hidden = true;
    this.root.setAttribute("role", "dialog");
    this.root.setAttribute("aria-label", "Player characteristics");

    this.titleEl = document.createElement("h2");
    this.bodyEl = document.createElement("div");
    this.bodyEl.className = "overlay-body";

    const hint = document.createElement("p");
    hint.className = "overlay-hint";
    hint.textContent = "Click empty space or press Esc to dismiss";

    this.root.append(this.titleEl, this.bodyEl, hint);
    host.append(this.root);
  }

  show(details: PlayerDetails, clientX: number, clientY: number): void {
    this.titleEl.textContent = `Player #${details.id}`;
    this.bodyEl.replaceChildren();

    const meta = document.createElement("dl");
    meta.className = "player-meta";
    const entries: Array<[string, string]> = [
      ["team", details.team],
      ["level", String(details.level)],
      ["position", `(${details.x}, ${details.y})`],
      ["orientation", details.orientation],
    ];
    for (const [key, value] of entries) {
      const dt = document.createElement("dt");
      dt.textContent = key;
      const dd = document.createElement("dd");
      dd.textContent = value;
      dd.dataset.field = key;
      meta.append(dt, dd);
    }
    this.bodyEl.append(meta);

    const invTitle = document.createElement("h3");
    invTitle.textContent = "inventory";
    this.bodyEl.append(invTitle);

    const list = document.createElement("ul");
    for (const row of details.inventory) {
      const li = document.createElement("li");
      li.dataset.resource = row.name;
      const swatch = document.createElement("span");
      swatch.className = "swatch";
      const name = row.name as ResourceName;
      swatch.style.background = ICON_SPECS[name].color;
      const label = document.createElement("span");
      label.className = "name";
      label.textContent = row.name;
      const count = document.createElement("span");
      count.className = "count";
      count.textContent = String(row.count);
      li.append(swatch, label, count);
      list.append(li);
    }
    this.bodyEl.append(list);

    this.root.dataset.text = formatPlayerDetailsText(details);
    this.root.hidden = false;
    positionPanel(this.root, clientX, clientY);
  }

  hide(): void {
    this.root.hidden = true;
  }

  get visible(): boolean {
    return !this.root.hidden;
  }

  get textSnapshot(): string {
    return this.root.dataset.text ?? "";
  }
}

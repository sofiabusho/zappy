/**
 * Floating square-detail overlay (G03).
 *
 * DOM panel (tooltip / floating window) showing per-resource counts so stone
 * numbers on a square are distinguishable (AQ16 / AQ17).
 */

import { ICON_SPECS } from "./icons.js";
import {
  formatTileDetailsText,
  type TileDetails,
} from "./tileDetails.js";

export class TileOverlay {
  private readonly root: HTMLElement;
  private readonly titleEl: HTMLElement;
  private readonly bodyEl: HTMLElement;

  constructor(host: HTMLElement) {
    this.root = document.createElement("aside");
    this.root.id = "tile-overlay";
    this.root.hidden = true;
    this.root.setAttribute("role", "dialog");
    this.root.setAttribute("aria-label", "Square details");

    this.titleEl = document.createElement("h2");
    this.bodyEl = document.createElement("div");
    this.bodyEl.className = "tile-overlay-body";

    const hint = document.createElement("p");
    hint.className = "tile-overlay-hint";
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
    players.className = "tile-overlay-players";
    players.textContent =
      details.playerIds.length > 0
        ? `players: ${details.playerIds.map((id) => `#${id}`).join(", ")}`
        : "players: (none)";
    this.bodyEl.append(players);

    this.root.dataset.text = formatTileDetailsText(details);
    this.root.hidden = false;
    this.position(clientX, clientY);
  }

  hide(): void {
    this.root.hidden = true;
  }

  get visible(): boolean {
    return !this.root.hidden;
  }

  /** Last plain-text snapshot (for tests / a11y). */
  get textSnapshot(): string {
    return this.root.dataset.text ?? "";
  }

  private position(clientX: number, clientY: number): void {
    const margin = 12;
    const panel = this.root;
    // Place then clamp so the panel stays on-screen.
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
}

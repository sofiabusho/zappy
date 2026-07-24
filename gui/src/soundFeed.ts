/**
 * DOM sound feed for broadcast / hear events (G05 / AQ20).
 */

import { type SoundEvent } from "./sound.js";

export class SoundFeed {
  private readonly root: HTMLElement;
  private readonly list: HTMLElement;

  constructor(host: HTMLElement) {
    this.root = document.createElement("aside");
    this.root.id = "sound-feed";
    this.root.setAttribute("aria-label", "Broadcast and sound feed");

    const title = document.createElement("h2");
    title.textContent = "Sounds";
    this.list = document.createElement("ul");
    this.list.id = "sound-feed-list";

    const empty = document.createElement("p");
    empty.className = "sound-feed-empty";
    empty.id = "sound-feed-empty";
    empty.textContent = "No broadcasts yet";

    this.root.append(title, empty, this.list);
    host.append(this.root);
  }

  /** Refresh the feed from active sound events (newest first). */
  update(events: SoundEvent[]): void {
    const empty = this.root.querySelector("#sound-feed-empty");
    if (empty instanceof HTMLElement) {
      empty.hidden = events.length > 0;
    }
    this.list.replaceChildren();
    const ordered = [...events].sort((a, b) => b.bornAt - a.bornAt);
    for (const ev of ordered.slice(0, 12)) {
      const li = document.createElement("li");
      li.dataset.kind = ev.kind;
      if (ev.k !== null) {
        li.dataset.k = String(ev.k);
      }
      const badge = document.createElement("span");
      badge.className = "sound-badge";
      if (ev.kind === "emit") {
        badge.textContent =
          ev.fromId !== null ? `pbc #${ev.fromId}` : "pbc";
      } else {
        badge.textContent =
          ev.toId !== null ? `K=${ev.k} → #${ev.toId}` : `K=${ev.k}`;
      }
      const text = document.createElement("span");
      text.className = "sound-text";
      text.textContent = ev.text;
      li.append(badge, text);
      this.list.append(li);
    }
  }
}

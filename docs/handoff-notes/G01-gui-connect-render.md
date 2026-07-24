# Handoff note — `G01` `gui-connect-render`

> Short turn summary. Not a PR.

## Summary

- Built the graphic client's connect path + live map render on plain HTML5
  Canvas 2D (no game engine): `transport → protocol parse → world → renderer`.
- Defined and documented the server→GUI side protocol (`msz`, `bct`, `tna`)
  in `docs/SDS.md` §12 and `gui/README.md`; player protocol untouched. Unknown
  verbs are ignored so G02–G05 can extend the stream.
- Two transports: `WebSocketTransport` (browser → WebSocket bridge → TCP
  server) and `ReplayTransport` (deterministic offline demo so the map renders
  with no server). Map model is toroidal (RQ03): tile lookups wrap on both
  axes.
- Added `npm run build` (`tsconfig.build.json` → `dist/` ESM); `index.html`
  loads `dist/app.js` as a module.

## Files touched

- `gui/src/protocol.ts` — resource order, `parseGuiLine`, `splitLines` (new)
- `gui/src/world.ts` — toroidal `WorldState` (new)
- `gui/src/renderer.ts` — Canvas grid + resource-presence render (new)
- `gui/src/transport.ts` — `Transport`, WebSocket + Replay (new)
- `gui/src/demo.ts` — offline demo stream (new)
- `gui/src/app.ts` — DOM bootstrap / wiring (new)
- `gui/index.html` — canvas + HUD (new)
- `gui/tsconfig.build.json`, `gui/package.json` — build script (new/edit)
- `gui/README.md`, `docs/SDS.md` §12 — protocol docs (edit)
- `docs/ticket-tracker.md` — G01 → ✅ (edit)

## How to verify

```bash
cd gui
npm install
npm run lint          # eslint + tsc --noEmit  → clean
npm test              # tsc --noEmit           → clean
npm run build         # tsc → dist/
python3 -m http.server 8090   # serve gui/
# open http://127.0.0.1:8090/index.html  → toroidal grid + resource dots render
```

Headless render check performed this turn (Chromium): `msz` → grid drawable,
`bct` tiles applied, HUD shows `12×12`, status `open`, ~17k non-background
pixels drawn, zero console errors.

- AQ / RQ IDs checked: **AQ15** (client connects — WebSocket-bridge path
  documented + offline demo — and displays the map; verified render), **RQ21**
  (real-time ≥2D Canvas map render, no game engine). Square-click details
  (AQ16/AQ17 → G03), entity icons (AQ18/AQ28 → G02), and sound viz
  (AQ20 → G05) are intentionally out of G01 scope.

## Risks / follow-ups

- Live server→GUI is not end-to-end yet: it needs (a) a server GUI emitter of
  `msz`/`bct` and (b) a `websocat`-style bridge. G01 owns the client + wire
  format; the server emitter is a future server-track concern, not a G01 dep.
- Rendering marks resource presence with a dot; per-stone icons and counts are
  G02/G03.

## Next suggested ticket

- `G02` — Icons: players, food, all stone types visible (Deps G01 ✅).

# Handoff note — `G02` `map-icons`

> Short turn summary. Not a PR.

## Summary

- Distinct Canvas 2D icons for food, all six stones (jade/peridot/amber/amethyst/garnet/ammolite), and players — no game engine.
- Extended server→GUI protocol with `pnw` / `ppo` / `pdi`; world tracks players; offline demo guarantees every stone + food + players.
- Legend strip under the map keeps types distinguishable (AQ18/AQ28).
- Unit tests via `node --test` (`gui/src/g02.test.ts`); `package.json` is `"type": "module"`.

## Files touched

- `gui/src/icons.ts` (new)
- `gui/src/g02.test.ts` (new)
- `gui/src/protocol.ts`, `gui/src/world.ts`, `gui/src/renderer.ts`, `gui/src/demo.ts`, `gui/src/app.ts`
- `gui/package.json`, `gui/README.md`
- `docs/SDS.md` §12
- `docs/ticket-tracker.md`
- `docs/handoff-notes/G02-map-icons.md`

## How to verify

```bash
cd gui
npm install
npm run lint
npm test
npm run build
python3 -m http.server 8090
# open http://127.0.0.1:8090/index.html  → icons + legend in offline demo
```

- AQ / RQ IDs checked:
  - **RQ21** — ≥2D Canvas icons for entities; no game engine
  - **AQ18** — players, stones, and food visible on the map (demo + renderer)
  - **AQ28** — all six stone types present and labeled in the legend / icon catalog

## Risks / follow-ups

- Server still needs to emit `pnw`/`ppo`/`pdi` on the GUI channel for live games; offline demo covers audit visibility now.
- Click overlays remain G03/G04.

## Next suggested ticket

- `G03` — Click square → floating details with counts
- or `G04` — Click player → characteristics overlay

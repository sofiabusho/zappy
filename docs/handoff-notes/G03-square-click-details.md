# Handoff note — `G03` `square-click-details`

> Short turn summary. Not a PR.

## Summary

- Click a map square → floating detail panel listing food + all six stone counts
  (including zeros) so stone numbers are distinguishable (AQ16 / AQ17).
- Shared grid layout/hit-test (`layout.ts`); selection highlight on the canvas;
  Esc / click-outside dismisses.
- Unit tests in `g03.test.ts` (details formatting + hit-testing).

## Files touched

- `gui/src/tileDetails.ts`, `gui/src/layout.ts`, `gui/src/overlay.ts` (new)
- `gui/src/g03.test.ts` (new)
- `gui/src/app.ts`, `gui/src/renderer.ts`, `gui/src/world.ts`
- `gui/index.html`, `gui/package.json`, `gui/README.md`
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/G03-square-click-details.md`

## How to verify

```bash
cd gui
npm run lint
npm test
npm run build
python3 -m http.server 8090
# open http://127.0.0.1:8090/index.html — click a square for the floating panel
```

- AQ / RQ IDs checked:
  - **RQ21** — square detail overlay on ≥2D Canvas client (no game engine)
  - **AQ16** — click square shows content via floating panel
  - **AQ17** — every stone type has an explicit numeric count on the panel

## Risks / follow-ups

- Player characteristic overlay remains **G04** (panel only lists player ids on the tile).
- Sound viz is **G05**.

## Next suggested ticket

- `G04` — Click player → characteristics overlay

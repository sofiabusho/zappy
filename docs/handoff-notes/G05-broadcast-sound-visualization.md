# Handoff note — `G05` `broadcast-sound-visualization`

> Short turn summary. Not a PR.

## Summary

- Canvas ripples for `pbc` emits + K-direction arrows / K=0 rings for `pic`
  hears; DOM sound feed lists recent messages (AQ20 / RQ21).
- Protocol parsers for `pbc` / `pic`; world stores timed sound events; demo
  stream includes sample broadcasts so offline viz works.
- Also ships the G04 `world.ts` inventory follow-up that was missing from the
  prior G04 push.

## Files touched

- `gui/src/sound.ts`, `gui/src/soundFeed.ts`, `gui/src/g05.test.ts` (new)
- `gui/src/protocol.ts`, `gui/src/world.ts`, `gui/src/renderer.ts`, `gui/src/app.ts`
- `gui/src/demo.ts`, `gui/index.html`, `gui/package.json`, `gui/README.md`
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/G05-broadcast-sound-visualization.md`

## How to verify

```bash
cd gui
npm run lint
npm test
npm run build
python3 -m http.server 8090
# open http://127.0.0.1:8090/index.html — ripples, K arrows, right-side feed
```

- AQ / RQ IDs checked:
  - **RQ21** — sound visualization included on Canvas client (no game engine)
  - **AQ20** — sounds are visualizable (ripples / K indicators / feed)

## Risks / follow-ups

- Live server must emit `pbc`/`pic` on the GUI channel (S12 computes K for
  players; GUI bridge must forward equivalents). Offline demo covers the viz.
- Graphic track complete; next core ticket is **I01**.

## Next suggested ticket

- `I01` — Scripted multi-client localhost smoke (server+2 AI)

# Handoff note — `G04` `player-click-characteristics`

> Short turn summary. Not a PR.

## Summary

- Click a player icon → floating characteristics panel (team, level, position,
  orientation, inventory) for AQ19 / RQ21.
- Protocol extended with `plv` / `pin`; world stores inventory; player hit-test
  wins over square clicks when the cursor is on an icon.
- Demo stream includes `pin`/`plv` so offline inspection works.

## Files touched

- `gui/src/playerDetails.ts`, `gui/src/g04.test.ts` (new)
- `gui/src/protocol.ts`, `gui/src/world.ts`, `gui/src/layout.ts`
- `gui/src/overlay.ts`, `gui/src/app.ts`, `gui/src/renderer.ts`, `gui/src/demo.ts`
- `gui/index.html`, `gui/package.json`, `gui/README.md`
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/G04-player-click-characteristics.md`

## How to verify

```bash
cd gui
npm run lint
npm test
npm run build
python3 -m http.server 8090
# open http://127.0.0.1:8090/index.html — click a player chevron
```

- AQ / RQ IDs checked:
  - **RQ21** — player characteristics overlay on Canvas client (no game engine)
  - **AQ19** — click player → floating window with characteristics

## Risks / follow-ups

- Live server must emit `pin`/`plv` on the GUI channel for full inventory/level;
  spawn defaults to 10 food / 0 stones until then.
- Sound viz remains **G05**.

## Next suggested ticket

- `G05` — Broadcast/sound visualization

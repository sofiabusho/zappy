# gui/

TypeScript + HTML5 Canvas graphic client for Zappy. **No game engines.**

G01: connect + map. G02: icons. G03: click square → resource counts.
**G04: click player → characteristics overlay** (team, level, position,
orientation, inventory). Sound viz is G05.

## Run

```bash
npm install
npm run build
python3 -m http.server 8090
# open http://127.0.0.1:8090/index.html
# click a player chevron → floating characteristics panel
# click a bare square → resource counts (G03)
```

Offline demo (no `?ws=`) includes players with `pin`/`plv` so characteristics
are inspectable without a live server.

## Lint / test

```bash
npm run lint
npm test        # G02 + G03 + G04
```

## Player characteristics (G04 / AQ19)

Clicking a player icon opens a floating panel with:

- id, team, level, map position, orientation (N/E/S/W)
- inventory counts for food + all six stones

Player hits win over square hits when the cursor is on an icon.

## Server → GUI protocol (subset)

| Line | Meaning |
|------|---------|
| `msz` / `bct` / `tna` | Map + teams (G01) |
| `pnw #<id> <X> <Y> <O> <L> <team>` | Player spawn |
| `ppo #<id> <X> <Y> <O>` | Move / reorient |
| `plv #<id> <L>` | Level update (G04) |
| `pin #<id> <X> <Y> <food> … <ammolite>` | Inventory + position (G04) |
| `pdi #<id>` | Remove player |

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md) §12.

# gui/

TypeScript + HTML5 Canvas graphic client for Zappy. **No game engines.**

G01: connect + map. G02: icons. G03: click square → resource counts.
G04: click player → characteristics. **G05: broadcast/sound visualization**
(ripples, K-direction arrows, sound feed).

## Run

```bash
npm install
npm run build
python3 -m http.server 8090
# open http://127.0.0.1:8090/index.html
# offline demo shows emit ripples + K arrows + right-side sound feed
# click a player chevron → characteristics (G04)
# click a bare square → resource counts (G03)
```

## Lint / test

```bash
npm run lint
npm test        # G02 + G03 + G04 + G05
```

## Sounds (G05 / AQ20)

When the server (or demo) emits:

- `pbc #<id> <text>` — ripple at the emitter’s tile + feed entry
- `pic #<id> <K> <text>` — arrow / rings on the listener for sector K (0..8)

K matches the subject sound ring (SDS §9). Same-tile broadcasts use `K=0`
(concentric rings). Events fade over a few seconds.

## Server → GUI protocol (subset)

| Line | Meaning |
|------|---------|
| `msz` / `bct` / `tna` | Map + teams (G01) |
| `pnw` / `ppo` / `pdi` | Players (G02) |
| `plv` / `pin` | Level + inventory (G04) |
| `pbc #<id> <text>` | Broadcast emit (G05) |
| `pic #<id> <K> <text>` | Listener hears from sector K (G05) |

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md) §12.

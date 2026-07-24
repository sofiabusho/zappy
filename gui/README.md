# gui/

TypeScript + HTML5 Canvas graphic client for Zappy. **No game engines.**

G01: connect + map render. G02: icons for players, food, and all six stones.
**G03: click a square → floating detail panel with per-resource counts** (AQ16 /
AQ17). Player characteristic overlays (G04) and sound viz (G05) come next.

## Run

```bash
npm install
npm run build                 # tsc → dist/ (ESM)
python3 -m http.server 8090   # or any static file server, from gui/
# open http://127.0.0.1:8090/index.html
# click any map square → floating panel lists food + each stone count
```

With no query string the client runs an **offline demo** stream so the map
renders without a server (food + every stone type + players). To watch a live
game, put a WebSocket bridge in front of the server's GUI channel and pass its
URL (browsers cannot open raw TCP):

```bash
# bridge raw TCP <-> WebSocket
websocat --binary ws-l:127.0.0.1:8090 tcp:127.0.0.1:<gui-port>
# then open:  http://127.0.0.1:8090/index.html?ws=ws://127.0.0.1:8090
```

## Lint / test

```bash
npm run lint    # eslint + tsc --noEmit
npm test        # tsc + node --test (G02 + G03)
```

## Square details (G03)

Click a tile to open a floating panel that lists **every** resource with its
count (`food` and all six stones, including zeros) so stone numbers on that
square are distinguishable. Esc or click outside the grid dismisses it.

## Server → GUI protocol (this client's subset)

Line-based, `\n`-terminated ASCII. This is a documented **side channel**; it
never alters the AI player protocol (`docs/SDS.md` §3). Resource order matches
the inventory contract: `food jade peridot amber amethyst garnet ammolite`.

| Line | Meaning |
|------|---------|
| `msz <X> <Y>` | Map size (width, height). Grid becomes drawable. |
| `bct <X> <Y> <food> <jade> <peridot> <amber> <amethyst> <garnet> <ammolite>` | Tile content (counts). |
| `tna <name>` | A team name. |
| `pnw #<id> <X> <Y> <O> <L> <team>` | Player appears (orientation 1=N…4=W, level, team). |
| `ppo #<id> <X> <Y> <O>` | Player position / facing update. |
| `pdi #<id>` | Player dies / disconnects — remove icon. |

Unknown verbs (future broadcast events for G05) are ignored.

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md) §12.

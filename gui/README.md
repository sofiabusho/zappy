# gui/

TypeScript + HTML5 Canvas graphic client for Zappy. **No game engines.**

G01 delivers the connect path and the live map render (toroidal grid +
resource presence). Icons per stone type (G02), click-to-inspect a square
(G03) and a player (G04), and sound/broadcast visualization (G05) build on
this foundation.

## Run

```bash
npm install
npm run build                 # tsc → dist/ (ESM)
python3 -m http.server 8090   # or any static file server, from gui/
# open http://127.0.0.1:8090/index.html
```

With no query string the client runs an **offline demo** stream so the map
renders without a server. To watch a live game, put a WebSocket bridge in
front of the server's GUI channel and pass its URL (browsers cannot open raw
TCP):

```bash
# bridge raw TCP <-> WebSocket
websocat --binary ws-l:127.0.0.1:8090 tcp:127.0.0.1:<gui-port>
# then open:  http://127.0.0.1:8090/index.html?ws=ws://127.0.0.1:8090
```

## Lint / test

```bash
npm run lint    # eslint + tsc --noEmit
npm test        # tsc --noEmit
```

GUI behavior is additionally verified against the manual checklist in
`docs/handoff-notes/G01-gui-connect-render.md`.

## Server → GUI protocol (this client's subset)

Line-based, `\n`-terminated ASCII. This is a documented **side channel**; it
never alters the AI player protocol (`docs/SDS.md` §3). Resource order matches
the inventory contract: `food jade peridot amber amethyst garnet ammolite`.

| Line | Meaning |
|------|---------|
| `msz <X> <Y>` | Map size (width, height). Grid becomes drawable. |
| `bct <X> <Y> <food> <jade> <peridot> <amber> <amethyst> <garnet> <ammolite>` | Tile content (counts). |
| `tna <name>` | A team name. |

Unknown verbs (future player/broadcast events for G02–G05) are ignored, so the
client tolerates a fuller stream than it renders today.

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md) §12.

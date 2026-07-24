# server/

Rust TCP game server for Zappy.

Entrypoint: `./server/server` — builds the Rust binary (`zappy-server`) and runs it.
It parses `-p <port> -x <width> -y <height> -n <team> [<team> ...] -c <nb> [-t <t>]`
(S01). Bad or missing args print the subject usage and exit non-zero; `-t` defaults
to `100`. On a valid line it binds `127.0.0.1:<port>` with a multiplexed
non-blocking event loop (`mio`), sends `WELCOME\n`, and completes the team
handshake (S02/S03): known team → `nb-client\n` + `X Y\n`; unknown team →
`Error: the team <name> doesn't exist` on the server and disconnect.
On start it also builds a toroidal resource map (S04) and, on each successful
team join, spawns a player at level 1 with 1260 life TU and 0 stones (S05).
After handshake it accepts pipelined commands with `t`-based delays and a max-10
queue; unknown commands get `ko` (S06).

```bash
./server/server -p 8080 -x 10 -y 10 -c 5 -n team1 team2 -t 100
# other terminal: telnet 127.0.0.1 8080  →  WELCOME, then type team1
./server/server            # no args → usage, exit 1
```

## Lint / test

```bash
cd server
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Or from repo root: `./scripts/check.sh`

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md).

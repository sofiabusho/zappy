# server/

Rust TCP game server for Zappy.

Entrypoint: `./server/server` — builds the Rust binary (`zappy-server`) and runs it.
It parses `-p <port> -x <width> -y <height> -n <team> [<team> ...] -c <nb> [-t <t>]`
(S01). Bad or missing args print the subject usage and exit non-zero; `-t` defaults
to `100`. The TCP event loop lands in later server tickets.

```bash
./server/server -p 8080 -x 10 -y 10 -c 5 -n team1 team2 -t 100
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

# server/

Rust TCP game server for Zappy.

Entrypoint (stub): `./server/server` — prints subject usage until S01 implements real CLI.

## Lint / test

```bash
cd server
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Or from repo root: `./scripts/check.sh`

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md).

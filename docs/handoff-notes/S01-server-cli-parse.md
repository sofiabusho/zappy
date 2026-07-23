# Handoff note — `S01` `server-cli-parse`

> Short turn summary. Not a PR.

## Summary

- Replaced the shell usage stub with a real Rust CLI parser (`server/src/cli.rs`)
  for `-p -x -y -n -c [-t]`; `-t` defaults to `100` (RQ10/RQ17).
- Added the `zappy-server` binary (`server/src/main.rs`): valid args echo the
  resolved config and exit 0; missing/invalid args print the subject usage
  (matching `docs/raw/audit.md` line-for-line) and exit non-zero (AQ01).
- Rewired `server/server` to build and exec the binary; removed `server/stub.sh`.

## Files touched

- `server/src/cli.rs` (new — parser, `Config`, `CliError`, 17 unit tests)
- `server/src/main.rs` (new — binary entrypoint)
- `server/src/lib.rs` (expose `pub mod cli;`)
- `server/Cargo.toml` (add `[[bin]] zappy-server`)
- `server/server` (build+exec wrapper), `server/README.md`
- `server/stub.sh` (deleted)
- `docs/ticket-tracker.md` (S01 ✅; S02/S04 → 🟡; counts, who's-up)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./server/server                                        # usage, exit 1
./server/server -p 8080 -x 10 -y 10 -c 5 -n team1 team2 -t 10   # config, exit 0
./server/server -p 8080 -x 10 -y 10 -c 5 -n solo       # t defaults to 100
python3 scripts/test_wrappers.py                       # AQ01 usage lines
```

- AQ / RQ IDs checked: **RQ17** (all six flags parsed, `-t` optional/default 100),
  **AQ01** (`./server` prints subject usage, exits non-zero).

## Risks / follow-ups

- Validation choices beyond the subject (reject duplicate flags/teams, port 0,
  non-positive x/y/c/t) are conservative; revisit if a later ticket needs looser
  parsing. Team names starting with `-` are treated as flags.
- Wrapper builds via `cargo build --release` on each run (first call compiles).

## Next suggested ticket

- `S02` — multiplexed TCP listen + `WELCOME\n` (Deps S01 ✅), or `S04` (world/resources).

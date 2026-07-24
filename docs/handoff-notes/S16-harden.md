# Handoff note — `S16` `harden`

> Short turn summary. Not a PR.

## Summary

- Bind conflict → stderr `ERROR : Address already in use` (AQ05).
- No POSIX exec family / `std::process::Command` in `server/src/*.rs` (AQ03);
  wrapper runs the binary without shell `exec`.
- Connection-flood integration test stands in for local `siege` (AQ04); loop
  stays responsive and still handshakes afterward.
- **Server track S01–S16 is complete.**

## Files touched

- `server/src/harden.rs` (new)
- `server/src/main.rs`, `server/src/lib.rs`, `server/src/net.rs` (tests)
- `server/server` (drop shell `exec`)
- `server/README.md`, `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S16-harden.md`

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# harden::tests::*
# net::tests::second_bind_reports_address_already_in_use
# net::tests::survives_connection_flood_then_still_handshakes
```

Manual: start server, second `./server` same `-p` → `ERROR : Address already in use`.
Optional: `siege -b 127.0.0.1:<port>` against own localhost only.

- AQ / RQ IDs checked: **RQ16**, **AQ03**, **AQ04**, **AQ05**.

## Next suggested ticket

- `C01` AI client CLI/handshake, or `G01` GUI scaffold (check Deps).

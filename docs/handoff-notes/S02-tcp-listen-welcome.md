# Handoff note — `S02` `tcp-listen-welcome`

> Short turn summary. Not a PR.

## Summary

- Added a multiplexed non-blocking TCP event loop (`mio`) in `server/src/net.rs`:
  bind `127.0.0.1:<port>`, accept clients, send `WELCOME\n` (partial writes buffered).
- Wired `main` to start the loop after a successful CLI parse instead of exiting.
- Inbound bytes are drained (not parsed); full handshake remains S03.
- Poll uses a 50ms timeout so the loop never hangs forever (RQ16).

## Files touched

- `server/src/net.rs` (new — accept loop, WELCOME, unit/integration tests)
- `server/src/main.rs` (start `net::serve` after config echo)
- `server/src/lib.rs` (`pub mod net`)
- `server/Cargo.toml` / `Cargo.lock` (`mio` dependency)
- `server/README.md`, `docs/SDS.md` (net module note)
- `docs/ticket-tracker.md` (S02 ✅; S03 → 🟡; counts, who’s-up)
- `docs/handoff-notes/S02-tcp-listen-welcome.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./server/server -p 8080 -x 10 -y 10 -c 5 -n my_team -t 10
# other terminal:
telnet 127.0.0.1 8080   # expect WELCOME
```

- AQ / RQ IDs checked: **RQ16** (multiplexed TCP via `mio`, non-blocking poll with
  timeout, no per-connection hang), **AQ02** (TCP client receives `WELCOME\n` —
  unit tests + live smoke on port 18080).

## Risks / follow-ups

- Listen address is `127.0.0.1` only (localhost subject). Switch to `0.0.0.0` only
  if a later ticket needs it.
- Token reuse after `wrapping_add` is theoretically possible on very long runs;
  fine for S02, revisit if connection churn becomes huge.
- Bind-conflict messaging / siege hardening remains **S16**.

## Next suggested ticket

- `S03` — handshake team → nb-client → `x y` (Deps S02 ✅), or `S04` (world/resources).

# Handoff note — `I01` `multi-client-smoke`

> Short turn summary. Not a PR.

## Summary

- Added a scripted localhost smoke that boots the Rust server on an ephemeral
  port and launches **two** autonomous `./client/client` processes for one team,
  proving the server + AI client(s) wire together end-to-end (RQ01).
- Each client runs with `stdin` closed and only speaks to the server socket; a
  static scan asserts the client package has no client↔client IPC
  (`bind/listen/pipe/multiprocessing/subprocess`) — RQ20.
- RQ02 is verified structurally: the win detector uses the subject thresholds
  (6 players / level 8) and is called every tick of the live server loop
  (`net.rs` → `win::winning_team`). A real 6-at-level-8 win can't fire in a
  seconds-long smoke, and the single team runs `-c 5` so no win ends the server
  mid-run; the exhaustive win-logic cases stay in S15 (`win.rs` unit tests).
- Also checks the server keeps answering `WELCOME` to a fresh TCP connection
  while both AI clients play (multiplexed, non-blocking — RQ16 spirit).

## Files touched

- `scripts/test_smoke_multi_client.py` (new) — the smoke harness + assertions.
- `scripts/check.sh` — runs the new smoke in the scripts gate.
- `docs/ticket-tracker.md` — I01 → ✅; who's-up/next → I02; counts updated.

## How to verify

```bash
# Standalone (builds the release server if needed, ~12s):
python3 scripts/test_smoke_multi_client.py

# Or via the full gate:
./scripts/check.sh
```

- AQ / RQ IDs checked: RQ01 (server + AI client(s) + graphic client present and
  server+2 AI interacting), RQ02 (win condition wired at 6/level-8), RQ20 (no
  out-of-band client↔client data exchange).

## Risks / follow-ups

- The smoke depends on a release `cargo build`; first run is slower (cold build)
  — timeout is generous (90s) to absorb that.
- Test-method order matters: the responsiveness probe runs (alphabetically)
  before the method that terminates the clients — kept intentional.

## Next suggested ticket

- `I02` — walk all non-bonus AQs (AQ01–AQ33) and record evidence commands in a
  handoff note. Deps I01+G05+S16 are all ✅.

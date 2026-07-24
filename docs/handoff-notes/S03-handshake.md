# Handoff note — `S03` `handshake`

> Short turn summary. Not a PR.

## Summary

- Extended `server/src/net.rs` to finish the subject handshake after `WELCOME\n`:
  client sends `team-name\n`; server replies `nb-client\n` then `X Y\n`.
- Unknown team prints `Error: the team <name> doesn't exist` and disconnects
  (RQ19 / AQ14). Full teams are dropped without a handshake reply.
- `nb-client` is remaining free slots **after** the join (`-c` minus joined).
- Slots are restored when a joined client disconnects. Telnet `\r\n` accepted.

## Files touched

- `server/src/net.rs` (handshake state machine, team slots, tests)
- `server/src/main.rs`, `server/src/lib.rs` (comments)
- `server/README.md`, `docs/SDS.md` (handshake notes)
- `docs/ticket-tracker.md` (S03 ✅; S05/S06 → 🟡 where deps allow; counts)
- `docs/handoff-notes/S03-handshake.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./server/server -p 8080 -x 10 -y 10 -c 5 -n my_team -t 10
# terminal A: telnet 127.0.0.1 8080 → WELCOME → type my_team → expect e.g. 4 then 10 10
# terminal B: same with wrong_team → server prints Error: the team wrong_team doesn't exist; client kicked
```

- AQ / RQ IDs checked: **RQ19** (WELCOME / team / nb-client / x y; bad team
  errors out), **AQ14** (exact error string + client disconnected — live smoke).

## Risks / follow-ups

- Joined connections still discard inbound command bytes until S06.
- Player spawn inventory/level is **S05** (also needs S04 world).
- GUI/special team names are not special-cased yet (G01 ownership).

## Next suggested ticket

- `S04` — toroidal world + resource generator (Deps S01 ✅), or `S06` (time/queue).

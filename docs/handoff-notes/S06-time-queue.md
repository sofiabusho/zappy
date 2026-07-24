# Handoff note — `S06` `time-queue`

> Short turn summary. Not a PR.

## Summary

- Added `server/src/time.rs`: `action_duration(t, cost)` implements `cost/t`
  seconds (RQ10).
- Added `server/src/commands.rs`: subject command parse, delay table, per-player
  `CmdQueue` capped at 10 (RQ11/RQ12); unknown/malformed → immediate `ko`.
- Wired the net loop to ingest commands after handshake, schedule completions
  with adaptive poll timeout, and send stub/real replies (`connect_nbr`,
  `inventory` real; movement `ok` stub; pick/drop `ko` until S09).

## Files touched

- `server/src/time.rs`, `server/src/commands.rs` (new)
- `server/src/player.rs` (queue + inventory reply)
- `server/src/net.rs`, `server/src/lib.rs`
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S06-time-queue.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./server/server -p 8080 -x 10 -y 10 -c 5 -n my_team -t 100
# telnet: WELCOME → my_team → connect_nbr / inventory / advance / garbage
```

- AQ / RQ IDs checked: **RQ10**, **RQ11** (parse + delays + `ko`), **RQ12**
  (queue cap unit test), **AQ06** (timing skeleton via duration helpers + delayed
  `advance`/`inventory` integration tests). Full movement timing remains S07.

## Risks / follow-ups

- `advance`/`left`/`right` reply `ok` without moving yet (**S07**).
- `see` returns `{}` until **S08**; pick/drop always `ko` until **S09**.
- Enchantment only emits `evolution in progress` (**S14** for real ritual).

## Next suggested ticket

- `S07` — toroidal movement (Deps S05+S06 ✅).

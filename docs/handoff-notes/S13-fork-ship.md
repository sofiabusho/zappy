# Handoff note — `S13` `fork-ship`

> Short turn summary. Not a PR.

## Summary

- Also resolved the S12 merge: kept `broadcast.rs` (counter-clockwise sectors
  from `origin/main`), dropped the duplicate `sound.rs` path.
- `fork` (48/t) → `ok` and schedules a ship at the player's tile.
- After **600** TU the ship arrives: team `max`/`free` slots +1
  (`connect_nbr` rises); next join for that team spawns on the egg tile with
  a random facing.

## Files touched

- `server/src/eggs.rs` (new: incubate / hatch / spawn sites)
- `server/src/net.rs` (fork completion, `tick_ships`, `TeamSlots::add_slot`,
  spawn-at-egg on handshake)
- `server/src/player.rs` (`spawn_at`)
- `server/src/lib.rs`, `docs/SDS.md`, `docs/ticket-tracker.md`
- Merge cleanup: `docs/handoff-notes/S12-broadcast.md`

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# eggs::tests::*
# net::tests::fork_ship_increases_connect_nbr_after_600_tu
```

- AQ / RQ IDs checked: **RQ13**, **AQ26**.

## Risks / follow-ups

- Disconnect still restores slots up to current `max` (post-ship max included).
- Eggs are not shown on `see` tiles yet (optional / GUI later).

## Next suggested ticket

- `S14` enchantment ritual.

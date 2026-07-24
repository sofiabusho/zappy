# Handoff note — `S10` `hunger-death`

> Short turn summary. Not a PR.

## Summary

- Game clock drains 1 life TU per elapsed time unit (`elapsed_seconds * t`).
- At 0 life the server pushes `death\n` and disconnects the player (slot freed).
- Eating already adds 126 life TU via `pick food` (S09); unit tests cover
  starvation and extended survival.

## Files touched

- `server/src/time.rs` (`elapsed_time_units`)
- `server/src/player.rs` (`tick_life` + tests)
- `server/src/net.rs` (`DEATH`, hunger tick in event loop)
- `docs/SDS.md`, `docs/ticket-tracker.md` (S10 ✅; S14 → 🟡)
- `docs/handoff-notes/S10-hunger-death.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# player::tests::tick_life_starves_at_zero
# player::tests::eating_extends_life_before_starvation
# time::tests::elapsed_time_units_scales_with_t
```

- AQ / RQ IDs checked: **RQ07**, **AQ07** (starve→death), **AQ08** (eating
  extends life), **AQ30** (126 TU per food).

## Risks / follow-ups

- Full starting life (1260 TU) takes a while to starve at default `t=100`
  (~12.6s); use high `t` or unit tests for audits.
- Resource `respawn_tick` still not scheduled from the game clock.

## Next suggested ticket

- `S11` kick, `S12` broadcast, `S13` fork, or `S14` ritual.

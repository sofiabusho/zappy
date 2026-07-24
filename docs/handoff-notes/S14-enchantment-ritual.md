# Handoff note — `S14` `enchantment-ritual`

> Short turn summary. Not a PR.

## Summary

- Subject ritual table is exact in `server/src/ritual.rs` (AQ31).
- `enchantment` becomes active → `evolution in progress` to participants and
  `in_ritual` (kick blocked); ineligible → immediate `ko` (no 300 wait).
- After 300/t → consume stones, level += 1, `current level : K\n`.
- Stone pool = tile + participant inventories (tile caps vs multi-count rows).
- Player counts follow the **table** (1→2 is solo).
- Mid-ritual death leaving fewer than required players cancels → completion `ko`.

## Files touched

- `server/src/ritual.rs` (new)
- `server/src/commands.rs` (`peek_active`, `abort_active`)
- `server/src/net.rs` (begin/complete wiring, death clears rituals)
- `server/src/lib.rs`, `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S14-enchantment-ritual.md`

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# ritual::tests::*
```

- AQ / RQ IDs checked: **RQ09**, **AQ25**, **AQ31**.

## Risks / follow-ups

- S15 should detect win when 6 teammates hit level 8 after a successful ritual.
- Prose “two or more players” vs table row 1→2: we follow the table.

## Next suggested ticket

- `S15` win detection, or `S16` harden.

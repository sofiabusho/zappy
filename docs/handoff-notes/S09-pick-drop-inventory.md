# Handoff note — `S09` `pick-drop-inventory`

> Short turn summary. Not a PR.

## Summary

- Implemented `pick` / `drop` for `food` and the six stone types between the
  player's tile and inventory (`ok` / `ko`).
- Picking food adds 126 life TU; dropping food spends 126 and requires an empty
  food slot on the tile. Stone drops honor one-per-type and ≤3 kinds.
- `inventory` already returned the subject-shaped bag line (life TU as `food`).

## Files touched

- `server/src/player.rs` (pick/drop helpers + tests)
- `server/src/world.rs` (`StoneKind::parse`)
- `server/src/net.rs` (mutable world; wire pick/drop)
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S09-pick-drop-inventory.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# player::tests::pick_and_drop_food_and_stones
```

- AQ / RQ IDs checked: **RQ11** (pick/drop/inventory responses), **AQ23**
  (pick food), **AQ24** (pick stones).

## Risks / follow-ups

- Food is modeled only as life TU (no separate food count); dropping requires
  ≥126 remaining life.
- Hunger tick / `death` remain **S10**.

## Next suggested ticket

- `S10` — food consumption / death, or `S11` / `S12`.

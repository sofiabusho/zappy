# Handoff note — `S07` `movement`

> Short turn summary. Not a PR.

## Summary

- Implemented `advance` / `left` / `right` effects on `Player`:
  - `right` / `left` rotate facing 90° (N→E→S→W).
  - `advance` steps one tile in the facing direction and wraps on the torus
    (`+x` east, `+y` south).
- Wired effects into command completion so delayed `ok` replies apply real
  movement (RQ03 / RQ11 / AQ06 / AQ10).

## Files touched

- `server/src/player.rs` (turn/advance helpers + unit tests)
- `server/src/net.rs` (`complete_command` applies movement)
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S07-movement.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# unit: player::tests::advance_moves_forward_and_wraps_toroidally
# integration: net::tests::advance_left_right_reply_ok
```

- AQ / RQ IDs checked: **RQ03** (toroidal step), **RQ11** (advance/left/right →
  `ok`), **AQ06** (still via 7/t delay path), **AQ10** (right edge → left wrap
  unit test).

## Risks / follow-ups

- Facing at spawn is still random; auditors may ask about the N/E/S/W grid
  convention (`+y` = south) — documented in `Orientation::step_delta`.
- Players are not yet indexed on tiles for `see` / `kick` (**S08** / **S11**).

## Next suggested ticket

- `S08` — `see` vision, or `S09` — pick/drop/inventory.

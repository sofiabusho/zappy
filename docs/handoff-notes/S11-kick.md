# Handoff note — `S11` `kick`

> Short turn summary. Not a PR.

## Summary

- `kick` pushes every other player on the kicker's tile one step in the
  kicker's facing direction (toroidal wrap).
- Victims receive `moving <K>\n` where K is the push direction relative to
  the victim's facing (1 front, 3 right, 5 back, 7 left).
- Food/stones on the tile are never moved.
- Returns `ko` when alone on the tile, or when any occupant has `in_ritual`
  (flag ready for S14).

## Files touched

- `server/src/kick.rs` (new: `apply_kick`, `moving_k`, unit tests)
- `server/src/player.rs` (`in_ritual` flag)
- `server/src/net.rs` (complete kick + deliver `moving` to victims)
- `server/src/lib.rs` (export `kick` module)
- `docs/SDS.md` (kick K semantics)
- `docs/ticket-tracker.md` (S11 ✅)
- `docs/handoff-notes/S11-kick.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# kick::tests::*
```

- AQ / RQ IDs checked: **RQ14** (players only; ritual block; `moving <K>`).

## Risks / follow-ups

- S14 must set/clear `in_ritual` on participants for the ritual guard to
  matter in live play.
- Broadcast sound K (S12) is separate from kick `moving` K; both use the
  same sector numbering for cardinals.

## Next suggested ticket

- `S12` broadcast, `S13` fork, or `S14` ritual.

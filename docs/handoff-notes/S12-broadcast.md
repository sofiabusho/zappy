# Handoff note — `S12` `broadcast`

> Short turn summary. Not a PR.

## Summary

- `broadcast <text>` replies `ok` to the sender after 7/t.
- Every other player receives `message <K>,<text>\n`.
- K uses the shortest toroidal path from sender to receiver, mapped into
  sectors relative to the receiver's facing (`0` on the same tile):
  ```text
  8 1 2
  7 0 3
  6 5 4
  ```
- Sector layout is clockwise (north-up) so cardinals match kick (1/3/5/7).
  Raw text says "trigonometric / counterclockwise"; without `sound.png` we
  keep kick/common-diagram alignment.

## Files touched

- `server/src/sound.rs` (new: deltas, `sound_k`, broadcast targets, tests)
- `server/src/net.rs` (deliver messages on broadcast completion)
- `server/src/lib.rs` (export `sound`)
- `docs/SDS.md`, `docs/ticket-tracker.md` (S12 ✅)
- `docs/handoff-notes/S12-broadcast.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# sound::tests::*
```

- AQ / RQ IDs checked: **RQ15**, **AQ32** (`broadcast <text>`), **AQ33**
  (`message <K>,<text>` with correct K).

## Risks / follow-ups

- If an auditor insists on counterclockwise numbering from the missing
  `sound.png`, flip the sector map (and revisit kick K consistency).
- GUI sound viz (**G05**) and AI meetup (**C04**) depend on this.

## Next suggested ticket

- `S13` fork / connect_nbr, or `S14` ritual.

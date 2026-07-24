# Handoff note — `S12` `broadcast`

> Short turn summary. Not a PR.

## Summary

- Implemented `broadcast <text>`: sender gets `ok`; every *other* living player
  receives `message <K>,<text>\n`.
- Direction `K` follows the subject "Sound Transmission" rules: `0` on the same
  tile, else `1` = front and counter-clockwise around the receiver
  (front `1`, left `3`, back `5`, right `7`; diagonals `2/4/6/8`).
- `K` is computed against the **shortest toroidal path** (per-axis offset reduced
  to the representative nearest zero) rotated into each receiver's own facing.
- Broadcaster is excluded from delivery (players don't hear themselves).

## Files touched

- `server/src/broadcast.rs` (new) — `direction_k`, `shortest_delta`,
  `apply_broadcast` + unit tests.
- `server/src/lib.rs` — registered `broadcast` module.
- `server/src/net.rs` — wired `Command::Broadcast` in `tick_command_completions`
  (deliver `message K,text` to others, `ok` to sender); moved it out of the
  plain `ok` fallthrough; added two integration tests.
- `docs/ticket-tracker.md` — S12 → ✅, counts, who's-up, work queue.

## How to verify

```bash
cd server && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings
# 88 tests pass; fmt clean; clippy clean.
```

Manual (two telnet clients on the same team):
- Client A: `broadcast hi` → `ok`.
- Client B: receives `message <K>,hi` with a sector digit `0..8`.

- AQ / RQ IDs checked:
  - **RQ15** — `broadcast` → all others `message <K>,<text>` via shortest toroidal
    path. ✅ (`apply_broadcast` + `shortest_delta`).
  - **AQ32** — broadcast sent as `broadcast <text>`. ✅ (parser already accepted
    it in S06; sender path returns `ok`).
  - **AQ33** — server emits `message <K>,<text>` with correct K. ✅
    (counter-clockwise sectors, tested per facing).

## Risks / follow-ups

- Kick's `moving <K>` (S11) uses a **clockwise** odd-sector mapping
  (1 front, 3 right, 5 back, 7 left) as documented in SDS §9. Broadcast uses the
  subject's **counter-clockwise** numbering — these are intentionally different
  conventions; do not "unify" them without re-checking both specs.
- `sound.png` in `docs/raw/` was unavailable, so the diagonal ordering follows
  the textual "counter-clockwise from front" rule; revisit if the image implies
  a different diagonal split.

## Next suggested ticket

- `S13` — `fork` + ship timer + `connect_nbr` slots (Deps S05+S06 ✅).

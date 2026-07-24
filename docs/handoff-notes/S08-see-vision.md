# Handoff note — `S08` `see-vision`

> Short turn summary. Not a PR.

## Summary

- Added `server/src/vision.rs`: forward-triangle coords by level (`(L+1)^2`
  tiles), toroidal wrap, and `see` reply formatting.
- Tile contents: other `player`s, then `food`, then stones (subject order);
  viewer omitted from their own square; empty slots stay empty between commas.
- Wired `Command::See` completion to `vision::see_reply`.

## Files touched

- `server/src/vision.rs` (new)
- `server/src/lib.rs`, `server/src/net.rs`
- `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S08-see-vision.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# vision::tests::* — counts, growth, wrap, format, empty `{, , , }`
```

- AQ / RQ IDs checked: **RQ08** (vision triangle + `see` format), **AQ09**
  (sight grows with level — unit test compares level 1 vs 2 tile counts).

## Risks / follow-ups

- Stone order on a tile is jade→ammolite; subject examples permute freely.
- `see` still uses the 7/t queue path from S06.

## Next suggested ticket

- `S09` — pick / drop / inventory polish, or `S10` — hunger / death.

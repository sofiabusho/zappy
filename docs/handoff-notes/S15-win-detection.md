# Handoff note — `S15` `win-detection`

> Short turn summary. Not a PR.

## Summary

- Win when ≥6 living players on one team are at level 8 (RQ02).
- Checked each event-loop tick after gameplay updates.
- On win: log `server: team <name> wins (6 players at level 8)` and stop the
  serve loop. No invented AI-client victory line (subject has none).

## Files touched

- `server/src/win.rs` (new)
- `server/src/net.rs` (check + exit)
- `server/src/lib.rs`, `docs/SDS.md`, `docs/ticket-tracker.md`
- `docs/handoff-notes/S15-win-detection.md`

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# win::tests::*
```

- AQ / RQ IDs checked: **RQ02**.

## Risks / follow-ups

- GUI `seg` announcement is out of scope until a graphic protocol ticket.
- Last server track ticket: **S16** harden.

## Next suggested ticket

- `S16` — harden (no exec, bind conflict, stress sanity).

# Handoff note — `C05` `fork-ship`

> Short turn summary. Not a PR.

## Summary

- The gathering agent now grows the family when a **group ritual** (level ≥2) is
  short on same-level partners. Before rallying, a beacon polls `connect_nbr`;
  if **0** slots are free (no teammate can join), it sends `fork` to call a ship
  (RQ13). A positive count means a slot is already open, so it just holds the
  tile and waits.
- Forking yields to survival (skipped while hungry) and is throttled to at most
  once per `fork_period` cycles (default 30) so we never queue more ships than
  the family can fill — a ship takes 600/t to arrive.
- All coordination stays in-game (commands only); no side channel (RQ20).

## Files touched

- `client/zappy_client/gathering.py` (fork strategy: `_due_to_fork`,
  `_maybe_fork`; `fork_period` param + `_last_fork_cycle` state; rally hook +
  docstring)
- `client/tests/test_gathering.py` (6 new fork tests; isolated the pre-existing
  broadcast test with `fork_period=0`)
- `docs/ticket-tracker.md` (C05 → ✅; who's-up, next-queue, summary counts)

## How to verify

```bash
cd client && ruff check . && ruff format --check . && pytest
# fork tests: test_forks_when_beacon_short_on_partners_and_no_free_slot,
# test_skips_fork_when_a_slot_is_already_free, test_fork_disabled_falls_back_to_broadcast,
# test_fork_is_throttled_between_attempts, test_solo_ritual_never_forks,
# test_malformed_connect_nbr_is_non_fatal
```

- AQ / RQ IDs checked: **RQ13** (fork/ship + `connect_nbr` free slots — exact
  command syntax matches `server/src/commands.rs`; timings server-enforced in
  S13), **AQ26** (player calls a ship for a family slot).

## Risks / follow-ups

- Fork trigger is scoped to "beacon short on group-ritual partners". A broader
  strategy (pre-forking toward the 6×level-8 win before a ritual is even close)
  is possible later but out of C05's `S` scope.
- Level-1 (solo) rituals never fork, by design — no partners are needed.

## Next suggested ticket

- `G01` — GUI↔server connect path + render empty/live map (client track complete).

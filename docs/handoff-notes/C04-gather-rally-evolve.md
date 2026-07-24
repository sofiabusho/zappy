# Handoff note — `C04` `gather-rally-evolve`

> Short turn summary. Not a PR.

## Summary

- Extended the autonomous AI (C03 survival) into `GatheringAgent`: it now gathers
  the stone classes its current level's ritual needs, rallies same-level partners
  with `broadcast`, and performs the `enchantment` to level up — all via server
  commands only, so no data crosses between clients out of band (RQ20).
- Added a client-side ritual table (`zappy_client/ritual.py`) mirroring the
  subject / `server/src/ritual.rs`, asserted equal to the subject in tests
  (client half of AQ31). Level-ups are only ever granted by the server's
  `current level : K` push — the client uses the table just to decide what to
  gather and when to send `enchantment`.
- Meetup coordination: a player holding the required stones becomes a *beacon*
  (stays put, periodically broadcasts `zappy L<level>`); listeners parse the
  sound direction `K` and step toward it so the group converges on one tile.
- Verified live against the real server: one client gathered 4 stones, started
  the enchantment at level 1, and **evolved to level 2** (AQ25), then broadcast
  meetups for the level-2 (2-player) ritual.

## Files touched

- `client/zappy_client/ritual.py` — new: stone names + subject ritual table +
  `missing_stones` / `has_required_stones`.
- `client/zappy_client/gathering.py` — new: `GatheringAgent` (gather → rally →
  enchant), `approach_plan`, `meetup_text`/`is_meetup`.
- `client/zappy_client/pipeline.py` — `EventKind.RITUAL_LEVEL` + `parse_current_level`;
  `current level : K` classified as a push (never mistaken for a command reply).
- `client/zappy_client/parsing.py` — `nearest_with` (generalised `nearest_food`).
- `client/zappy_client/main.py` — entrypoint now drives `GatheringAgent`.
- `client/tests/test_ritual.py`, `client/tests/test_gathering.py` — new tests.
- `docs/ticket-tracker.md` — C04 → ✅; who's-up / counts / queue.

## How to verify

```bash
cd client && ruff check . && ruff format --check . && pytest   # 102 passing

# End-to-end (from repo root), t high so the ritual resolves fast:
./server/server -p 8099 -x 12 -y 12 -c 5 -n my_team -t 1000 &
./client/client -n my_team -p 8099 -h 127.0.0.1     # logs: picked …, enchantment started, evolved to level 2, broadcast meetup
```

- AQ / RQ IDs checked: RQ09 (ritual table exact + enchantment flow), RQ15
  (`broadcast <text>` sent, `message <K>,<text>` consumed & acted on), RQ20 (no
  out-of-band client↔client data), AQ25 (evolution ritual / level-up observed
  live).

## Risks / follow-ups

- A **lone** client cannot complete a ritual above level 1 (needs partners), so a
  single-client demo eventually starves while beaconing — expected, not a bug.
  Multi-client runs (via `fork`/C05 or several `./client`) enable group rituals.
- Meetup convergence is a coarse heuristic (turn-toward-`K`, advance); it relies
  on the beacon re-broadcasting until the group forms rather than exact pathing.
- Hunger gate uses the periodically-polled life gauge, so it can lag; tuning
  `hunger_threshold` / `inventory_period` could make long beacon waits safer.

## Next suggested ticket

- `C05` — fork strategy when the family needs slots (Deps C03 ✅ S13 ✅).

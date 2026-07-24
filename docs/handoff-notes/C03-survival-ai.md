# Handoff note — `C03` `survival-ai`

> Short turn summary. Not a PR.

## Summary

- Added `zappy_client/parsing.py`: `parse_see` (`{tile, ...}` → list of token
  lists) and `parse_inventory` (`{food N, ...}` → `{name: int}`, where `food` is
  remaining life TU, RQ07), plus vision geometry — `tile_offset(index)` maps a
  `see` index to (forward, lateral) using the `d²` triangle layout from
  `server/src/vision.rs`, and `nearest_food` finds the closest visible food.
- Added `zappy_client/survival.py`: `SurvivalAgent` — an autonomous perceive-act
  loop (see → eat food underfoot → walk to nearest visible food via `plan_toward`
  → wander when blind). Keeps **one command outstanding at a time**, so the ≤10
  pipeline window (RQ12) is never exceeded and responses pair unambiguously.
  Server pushes (`message`, `moving`) are skipped; `death` ends the run cleanly.
- Wired `main.py`: after the C01 handshake, `./client` now hands off to
  `SurvivalAgent` and plays with no human input (RQ18/RQ20). Uses only the game
  socket — no files, no side channels (RQ20).

## Files touched

- `client/zappy_client/parsing.py` (new)
- `client/zappy_client/survival.py` (new)
- `client/zappy_client/main.py` (idle stub → autonomous survival loop)
- `client/zappy_client/__init__.py` (docstring)
- `client/tests/test_parsing.py` (new — 19 tests)
- `client/tests/test_survival.py` (new — 19 tests)
- `docs/ticket-tracker.md` (C03 → ✅, Who's up, summaries, counts)

## How to verify

```bash
cd client
python -m pytest -q          # 72 passed (34 prior + 38 new)
ruff check .                 # All checks passed!
ruff format --check .        # all files formatted

# End-to-end (from repo root):
./server/server -p 8091 -x 10 -y 10 -c 5 -n my_team -t 500 &
timeout 6 ./client/client -n my_team -p 8091 -h 127.0.0.1
# client joins, plays autonomously, and its "life remaining" gauge climbs
# above the 1260 TU spawn as it eats — then exits cleanly on timeout/close.
```

- AQ / RQ IDs checked against `docs/raw/`:
  - **RQ07** (requirements §Food / §General Rules) — one food = 126 life TU;
    starvation kills. Client observes life via `inventory` (`food` = life TU) and
    eats food to extend it; a `death` push ends the run. Live run: life rose from
    1260 → 5300+ TU as the agent picked up food.
  - **RQ20** (requirements §Teams/Families / §Client-server) — clients must not
    exchange data outside the game. `SurvivalAgent` acts solely on server replies
    over the one TCP socket; no filesystem/IPC/client↔client channel.
  - **AQ12 / AQ13** (audit §Client) — `./client -n my_team -p 8091` and with
    `-h 127.0.0.1` both launch and interact without errors (verified live).
  - **AQ23** (audit §PLAYER) — player can pick up food: confirmed by the rising
    life gauge (each pick adds 126 TU) and `pick food` → `ok` on the wire.

## Risks / follow-ups

- Navigation is greedy per-cycle (walk the plan onto the nearest food, then
  re-see). If another player eats the target first, `pick food` returns `ko` and
  the next cycle simply re-perceives — no state corruption, just a wasted trip.
- `see`/`inventory` parsing lives in `parsing.py`; C04 should reuse `parse_see`
  for stone gathering and `tile_offset`/`plan_toward` for meetup navigation.
- The agent does not yet gather stones, broadcast, enchant, or fork — those are
  C04 (gather + meetup + ritual) and C05 (fork strategy).

## Next suggested ticket

- `C04` — Gathering + broadcast meetup + enchantment attempts (Deps C03 ✅,
  S12 ✅, S14 ✅).

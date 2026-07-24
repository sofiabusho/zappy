# Handoff note — `S05` `player-spawn`

> Short turn summary. Not a PR.

## Summary

- Added `server/src/player.rs`: starting loadout constants (level 1, 10 food →
  1260 life TU, 0 stones), `Player` / `Inventory` / `Orientation`, and
  `PlayerSet` registry.
- On successful team handshake the net loop spawns a player on a random tile
  with a random facing, logs the spawn, and removes them on disconnect.
- Team membership is stored on the player; slot accounting remains in `net`.

## Files touched

- `server/src/player.rs` (new)
- `server/src/lib.rs`, `server/src/net.rs` (spawn on join / despawn on drop)
- `server/README.md`, `docs/SDS.md`
- `docs/ticket-tracker.md` (S05 ✅; counts; who’s-up)
- `docs/handoff-notes/S05-player-spawn.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./server/server -p 8080 -x 10 -y 10 -c 5 -n my_team -t 10
# telnet 127.0.0.1 8080 → WELCOME → my_team
# stderr: server: player N team=my_team at X,Y level=1 life_tu=1260
```

- AQ / RQ IDs checked: **RQ06**, **AQ21** (10 food / 1260 TU, 0 stones),
  **AQ22** (level 1) — unit tests + spawn log.

## Risks / follow-ups

- Life is stored as `life_tu` (not a separate food count); S09 `inventory` should
  report `food` as remaining life TU per subject examples.
- Cmd queue / hunger tick remain **S06** / **S10**.
- Players are not yet listed on world tiles (see / kick later).

## Next suggested ticket

- `S06` — time scheduler + per-player cmd queue (Deps S03 ✅).

# Handoff note — `S04` `world-resources`

> Short turn summary. Not a PR.

## Summary

- Added `server/src/world.rs`: toroidal wrap helpers, six stone kinds + food
  tiles, seeded resource generation, and a documented respawn tick.
- Hard rules enforced: ≤1 food/tile, ≤1 of each stone type/tile, ≤3 stone kinds
  per tile, stones spread across the map (not piled on one square).
- Densities (food 0.50, jade→ammolite 0.30…0.05) and respawn (every 20 TU,
  refill 15% of missing) documented in module docs + `docs/SDS.md` (AQ29).
- Server startup generates a world and logs a resource summary line.

## Files touched

- `server/src/world.rs` (new)
- `server/src/lib.rs`, `server/src/net.rs` (wire world into serve loop)
- `server/README.md`, `docs/SDS.md`
- `docs/ticket-tracker.md` (S04 ✅; S05/G01 → 🟡; counts)
- `docs/handoff-notes/S04-world-resources.md` (this file)

## How to verify

```bash
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
./server/server -p 8080 -x 10 -y 10 -c 5 -n my_team -t 10
# stderr should include: server: world 10x10 seed=… food=… jade=… … ammolite=…
```

- AQ / RQ IDs checked:
  - **RQ03** — plains map; toroidal wrap API (`wrap_x` / `wrap_y`)
  - **RQ04** / **AQ28** — jade, peridot, amber, amethyst, garnet, ammolite
  - **RQ05** / **AQ29** — random generation with documented rules
  - **AQ10** — wrap tests (right→left); movement still S07
  - **AQ27** — food and stones present after generation

## Risks / follow-ups

- Respawn is implemented but not yet scheduled (needs S06 time loop).
- World is retained in the accept loop but unused by protocol until S05/S07/S09.
- Density CLI flag remains bonus **B03**.

## Next suggested ticket

- `S05` — player spawn state (Deps S03+S04 ✅), or `S06` — time/queue.

# SDS — Zappy

Technical design derived from `docs/raw/requirements.md`. When this conflicts with raw docs, **raw wins**.

## 1. System context

```text
┌─────────────┐     TCP      ┌──────────────────┐     TCP      ┌─────────────┐
│ AI client(s)│◄────────────►│     server       │◄────────────►│ graphic GUI │
│  ./client   │  player proto│  ./server        │  GUI proto*  │  (browser)  │
└─────────────┘              │  world + time    │              └─────────────┘
                             └──────────────────┘
```

\*GUI protocol: not fully specified in raw requirements. Implement a **documented** server↔GUI channel (dedicated team name, magic handshake, or side protocol) in tickets that own GUI connect; keep player protocol untouched.

## 2. Binaries & CLI contracts

### Server

```text
./server -p <port> -x <width> -y <height> -n <team> [<team> ...] -c <nb> [-t <t>]
```

- Default `t = 100` if omitted.
- Missing/invalid args → usage on stdout/stderr (match audit sample closely).
- Second process on same port → bind error (`Address already in use` or equivalent).

### AI client

```text
./client -n <team> -p <port> [-h <hostname>]
```

- Default host: `localhost`.

## 3. Player protocol (normative)

All lines end with `\n`.

### Handshake

| Direction | Payload |
|-----------|---------|
| S→C | `WELCOME` |
| C→S | `<team-name>` |
| S→C | `<nb-client>` then `<width> <height>` (`nb-client` = free slots **after** this join) |
| Error | Server prints `Error: the team <name> doesn't exist`; drop client |
| Full team | No handshake reply; drop client (slot restored only for prior successful joins on disconnect) |

### Commands

| Command | Delay | Response |
|---------|-------|----------|
| `advance` | 7/t | `ok` |
| `right` | 7/t | `ok` |
| `left` | 7/t | `ok` |
| `see` | 7/t | `{tile0, tile1, ...}` |
| `inventory` | 1/t | `{food N, jade N, ...}` |
| `pick <object>` | 7/t | `ok` / `ko` |
| `drop <object>` | 7/t | `ok` / `ko` |
| `kick` | 7/t | `ok` / `ko`; victims get `moving <K>` |
| `broadcast <text>` | 7/t | `ok`; others get `message <K>,<text>` |
| `enchantment` | 300/t | `evolution in progress` / then `current level : K` |
| `fork` | 48/t | `ok` (ship arrives after 600/t) |
| `connect_nbr` | 0/t | integer |
| (server push) | — | `death` |

- Unknown / malformed → `ko`.
- Per-client pending successful requests capped at **10**; further ignored until slots free.
- Clients may pipeline requests; server executes in receive order per player; delays block that player only.

## 4. Modules (server)

| Module | Responsibility |
|--------|----------------|
| `cli` | Arg parse, usage, defaults |
| `net` | Multiplexed accept/read/write via `mio` poll; send `WELCOME\n` on connect; never blocks the event loop forever (50ms poll timeout) |
| `world` | Toroidal grid; tile food/stones; density-based generate + respawn rules (S04) |
| `player` | Position, orientation, level, inventory, food timer, team, queue |
| `commands` | Parse, validate, enqueue, apply effects |
| `time` | Global tick from `t`; schedule action completions |
| `ritual` | Enchantment eligibility, consumption, level-up |
| `broadcast` | Shortest toroidal path → sound sector K ∈ {0..8} |
| `eggs` | Fork → ship timer → slot / connect_nbr |
| `win` | Detect ≥6 teammates at level 8 |
| `gui` (optional crate area) | Feed map/entity events to graphic client |

## 5. Data model (conceptual)

```text
World { width, height, tiles[y][x], teams[], t, tick }
Tile  { food: 0|1, stones: set≤3 distinct types, players[], eggs[] }
Team  { name, max_clients, slots_free, members[] }
Player {
  id, team, x, y, orient ∈ {N,E,S,W},
  level 1..=8,
  inventory: {food, jade, peridot, amber, amethyst, garnet, ammolite},
  food_units_remaining_time,  // 10 food → 1260 TU at start
  cmd_queue: VecDeque≤10,
  ritual_state
}
```

### Stones

`jade | peridot | amber | amethyst | garnet | ammolite`

### Resource generation rules (project-owned; must be explained to auditors)

Implemented in `server/src/world.rs` (S04). Subject hard constraints (always enforced):

1. At most **one food** per square.
2. At most **one** stone of each type per square.
3. At most **three** different stone types on one square.
4. Do not dump all stones onto a single square — placement shuffles the map and
   spreads each stone type independently.

**Initial densities** (fraction of tiles; `target = round(density × width × height)`):

| Resource | Density |
|----------|--------:|
| food | 0.50 |
| jade | 0.30 |
| peridot | 0.25 |
| amber | 0.20 |
| amethyst | 0.15 |
| garnet | 0.10 |
| ammolite | 0.05 |

**Algorithm:** for each resource, shuffle tile indices with a seeded xorshift RNG
and place on the first eligible tiles (respecting the hard constraints). Same
seed → same map (supports a future seed CLI flag).

**Respawn:** every `RESPAWN_PERIOD_TU` (20) time units, `World::respawn_tick`
tries to refill each resource toward the same densities at `RESPAWN_RATE` (15%
of the missing count per pass). Respawn is implemented now; the time loop will
call it from S06+.
## 6. Vision

Level `L` sees a forward triangle: row `d` (1..=L) has `2*d+1` tiles, indexed as in raw diagrams. `see` lists tile contents left-to-right, near-to-far; player does not see self; multiple objects on a tile are space-separated; empty tiles appear empty in the brace list.

## 7. Ritual table

| Level | Players | jade | peridot | amber | amethyst | garnet | ammolite |
|------:|--------:|-----:|--------:|------:|---------:|-------:|---------:|
| 1→2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 2→3 | 2 | 1 | 1 | 1 | 0 | 0 | 0 |
| 3→4 | 2 | 2 | 0 | 1 | 0 | 2 | 0 |
| 4→5 | 4 | 1 | 1 | 2 | 0 | 1 | 0 |
| 5→6 | 4 | 1 | 2 | 1 | 3 | 0 | 0 |
| 6→7 | 6 | 1 | 2 | 3 | 0 | 1 | 0 |
| 7→8 | 6 | 2 | 2 | 2 | 2 | 2 | 1 |

- Same **level** required; teams may mix.
- One player starts `enchantment`; others join on tile.
- If participants die mid-ritual and one remains alone → must restart.

## 8. Time

- Time unit duration = `1/t` seconds.
- Action wall time ≈ `cost / t` seconds.
- 1 food = **126** time units of life.

## 9. Sound direction K

- Same tile → `K = 0`.
- Else shortest path on torus; map arrival into sectors 1..8 (1 = front, then counterclockwise).
- See raw `sound.png` reference when available.

## 10. Directory contracts

| Path | Contract |
|------|----------|
| `server/` | Rust package producing server binary / root `./server` wrapper |
| `client/` | Python package producing `./client` |
| `gui/` | TS+Canvas app; README explains how to connect for auditors |
| `docs/raw/` | READ-ONLY acceptance |
| `docs/handoff-notes/` | Optional turn notes, not PRs |

## 11. AI client behavior (minimum)

Autonomous loop: maintain food → explore/see → pick resources → broadcast for meetup → enchantment when eligible → `fork` when slots needed / strategy requires. No external collusion.

## 12. Graphic client behavior (minimum)

- Connect and render map in real time.
- Icons for players, food, stones.
- Click tile → floating details (counts per resource).
- Click player → characteristics overlay.
- Visualize broadcasts/sounds.

## 13. Testing map

| Concern | Suggested tests |
|---------|-----------------|
| Torus wrap | move off edge → opposite edge |
| Vision sizes | level 1/2/3 tile counts |
| Ritual table | lookup exact |
| Food clock | 126 TU per unit; death |
| Sound K | same tile 0; known geometry cases |
| Queue | 11th command ignored |
| Handshake | welcome, bad team, nb-client |

## 14. Examples

### Inventory response

```text
{food 300, amber 4, garnet 7, peridot 2, jade 0, amethyst 0, ammolite 0}
```

### See (level 1 sample from subject)

```text
{food, player amber, garnet garnet, }
```

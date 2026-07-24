# Ticket tracker — Zappy

**Legend:** 🔴 Blocked · 🟡 Ready · 🟢 In Progress · ✅ Done · ⬜ Not Started

## Turn protocol

1. `git pull` on `main`
2. Pick **one** ticket that is 🟡 or ⬜ with **all Deps ✅**
3. Set status → 🟢 and fill **Claimed by**
4. Implement **only** that ticket (paste `docs/AGENT_WORKFLOW.md`)
5. Verify (tests/lint + listed RQ/AQ vs `docs/raw/`) → set ✅ → push `main`
6. Next person starts at step 1

---

## 1. Scope contract

| Priority | Source | Role |
|----------|--------|------|
| 1 | `docs/raw/requirements.md` | Must-build behavior |
| 2 | `docs/raw/audit.md` | Pass/fail auditor checks |
| 3 | Server + AI client + graphic client | Primary deliverable |
| 4 | Bonus (3D, log, density, seed) | Optional; do after core AQs |

## 2. Who’s up / last push

| Field | Value |
|-------|-------|
| Last push | S11 kick — agent |
| Who’s up | Next: claim **S12**, **S13**, or **S14** (ritual) |
| Note | Serial turns only; do not start a second 🟢 |

## 3. Tracks

| Track | Letter | Focus |
|-------|--------|-------|
| Bootstrap | A | Scaffold, lint, test, README, wrappers |
| Server | S | TCP world engine |
| AI client | C | Autonomous `./client` |
| Graphic | G | 2D visualizer |
| Integration | I | E2E + audit dry-run |
| Bonus | B | Optional flags / 3D |

Tracks are **focus areas**, not parallel merge lanes.

## 4. ID legends (raw files are unnumbered)

### Requirements (RQ*) — from `docs/raw/requirements.md`

| ID | Testable statement |
|----|--------------------|
| RQ01 | System has server, AI client(s), and graphic client |
| RQ02 | Win when ≥6 members of one team reach level 8 |
| RQ03 | World is obstacle-free plains; map is toroidal |
| RQ04 | Six stone types: jade, peridot, amber, amethyst, garnet, ammolite |
| RQ05 | Resources randomly generated under documented logical rules |
| RQ06 | Players start with 10 food, 0 stones, level 1 |
| RQ07 | 1 food = 126 time units of life; starvation kills |
| RQ08 | Vision expands with level per subject diagrams; `see` format correct |
| RQ09 | Evolution ritual uses exact player/stone table; enchantment rules honored |
| RQ10 | Time unit = 1/t s; action duration = cost/t; default t=100 |
| RQ11 | Commands/syntax/delays/responses match subject table; unknown → `ko` |
| RQ12 | Per-player request buffer max 10 |
| RQ13 | `fork` 48/t + ship 600/t; `connect_nbr` reports free slots |
| RQ14 | `kick` moves players (not resources); blocked during ritual; `moving <K>` |
| RQ15 | `broadcast` → all others `message <K>,<text>` via shortest toroidal path |
| RQ16 | Server in C/C++/Rust; multiplexed TCP; no `exec*`; never hangs forever |
| RQ17 | Server CLI: `-p -x -y -n -c [-t]` |
| RQ18 | Client CLI: `-n -p [-h]`; default host localhost; autonomous |
| RQ19 | Handshake: WELCOME / team / nb-client / x y; bad team errors out |
| RQ20 | Clients must not exchange data outside the game |
| RQ21 | Graphic: ≥2D icons, no game engine; show entities; square details; sound viz |
| RQB1 | *(bonus)* 3D or alternate representation |
| RQB2 | *(bonus)* Server log-mode flag |
| RQB3 | *(bonus)* Resource/food density flag |
| RQB4 | *(bonus)* Seed flag for reproducible scenarios |

### Audit questions (AQ*) — from `docs/raw/audit.md`

| ID | Yes/no question (condensed) |
|----|------------------------------|
| AQ01 | `./server` prints usage like the subject? |
| AQ02 | Telnet receives `WELCOME`? |
| AQ03 | No `exec*` family used? |
| AQ04 | Server still works under local `siege` stress? |
| AQ05 | Second `./server` same port → address-in-use style error? |
| AQ06 | Movement/action timings respected? |
| AQ07 | No food → starve and die? |
| AQ08 | Eating extends survival? |
| AQ09 | Sight increases with level? |
| AQ10 | Leaving right edge re-enters left (torus)? |
| AQ11 | `./client` prints usage? |
| AQ12 | Client launches vs server without errors? |
| AQ13 | Client `-h 127.0.0.1` interacts cleanly? |
| AQ14 | Wrong team → server error + client kicked? |
| AQ15 | Graphic connects and displays map? |
| AQ16 | Click square shows content (overlay)? |
| AQ17 | Stone counts distinguishable on a square? |
| AQ18 | Players, stones, food visible on map? |
| AQ19 | Click player shows characteristics overlay? |
| AQ20 | Sounds visualizable? |
| AQ21 | Start: 10 food (1260 TU) and 0 stones? |
| AQ22 | Start level 1? |
| AQ23 | Can pick up food? |
| AQ24 | Can pick up stones? |
| AQ25 | Can perform evolution ritual / level up? |
| AQ26 | Can call ship / fork for family slot? |
| AQ27 | Food and stones exist as resources? |
| AQ28 | All six stone types present? |
| AQ29 | Random generation with clear explained rules? |
| AQ30 | One food = 126 time units? |
| AQ31 | Ritual rules match subject table? |
| AQ32 | Broadcast sent as `broadcast <text>`? |
| AQ33 | Server emits `message <K>,<text>` with correct K? |
| AQB1 | *(bonus)* Visualizer in 3D? |
| AQB2 | *(bonus)* Log-mode flag? |
| AQB3 | *(bonus)* Density flag? |
| AQB4 | *(bonus)* Seed flag? |

## Assumptions

1. **Server language:** Rust (allowed by subject).
2. **AI client:** Python 3.11+.
3. **GUI:** TypeScript + HTML5 Canvas; no game engine.
4. **GUI↔server protocol:** not fully specified in raw; owned by G01/SDS — must not break player protocol.
5. **Resource respawn:** exact rates chosen in S04 and documented for auditors (AQ29).
6. **Wrappers:** root `./server` and `./client` created in Sprint 0 / early tickets.

---

## 5–6. Sprints & tickets

### Sprint 0 — Bootstrap / guardrails

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| A01 | ✅ | Create repo scaffold: `server/`, `client/`, `gui/`, `scripts/`, root README pointing at AGENTS.md | S | — | RQ01 | agent |
| A02 | ✅ | Add `./server` and `./client` wrapper stubs + placeholder usage strings matching subject | S | A01 | RQ17, RQ18, AQ01, AQ11 | agent |
| A03 | ✅ | Wire lint/format/test commands (`cargo fmt/clippy/test`, `ruff`/`pytest`, GUI lint) + `scripts/check.sh` | M | A01 | — | agent |
| A04 | ✅ | Document build/run/audit quickstart in README (localhost only; siege warning) | S | A02, A03 | RQ16 | agent |

### Sprint 1 — Server foundation

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| S01 | ✅ | CLI parse `-p -x -y -n -c -t`; usage on bad/missing args; default t=100 | M | A02 | RQ17, AQ01 | routine |
| S02 | ✅ | Multiplexed TCP listen; accept; send `WELCOME\n`; non-blocking event loop skeleton | M | S01 | RQ16, AQ02 | agent |
| S03 | ✅ | Handshake: team → nb-client → `x y`; invalid team error + disconnect | M | S02 | RQ19, AQ14 | agent |
| S04 | ✅ | Toroidal world + resource generator with documented rules; six stone types + food | L | S01 | RQ03, RQ04, RQ05, AQ10, AQ27, AQ28, AQ29 | agent |
| S05 | ✅ | Player spawn state: level 1, 10 food→1260 TU, 0 stones; team membership | M | S03, S04 | RQ06, AQ21, AQ22 | agent |
| S06 | ✅ | Time scheduler (`t`) + per-player cmd queue (max 10) + unknown→`ko` | L | S03 | RQ10, RQ11, RQ12, AQ06 | agent |

### Sprint 2 — Server gameplay

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| S07 | ✅ | `advance` / `left` / `right` with delays; toroidal movement | M | S05, S06 | RQ03, RQ11, AQ06, AQ10 | agent |
| S08 | ✅ | `see` vision by level + response formatting | M | S07 | RQ08, AQ09 | agent |
| S09 | ✅ | `inventory`, `pick`, `drop` | M | S07 | RQ11, AQ23, AQ24 | agent |
| S10 | ✅ | Food consumption over time; `death`; eating extends life (126 TU) | M | S05, S06 | RQ07, AQ07, AQ08, AQ30 | agent |
| S11 | ✅ | `kick` + `moving <K>`; no kick during ritual; resources unaffected | M | S07 | RQ14 | agent |
| S12 | 🟡 | `broadcast` + directional `message <K>,<text>` (shortest path) | L | S07 | RQ15, AQ32, AQ33 | |
| S13 | 🟡 | `fork` + ship timer + `connect_nbr` slots | M | S05, S06 | RQ13, AQ26 | |
| S14 | 🟡 | `enchantment` / ritual table + mid-ritual alone restart | L | S09, S10 | RQ09, AQ25, AQ31 | |
| S15 | ⬜ | Win detection: 6 teammates at level 8 | S | S14 | RQ02 | |
| S16 | ⬜ | Harden: no exec paths; bind conflict message; local stress sanity | M | S02 | RQ16, AQ03, AQ04, AQ05 | |

### Sprint 3 — AI client

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| C01 | ⬜ | Real `./client` CLI + TCP connect + handshake | M | S03, A02 | RQ18, RQ19, AQ11, AQ12, AQ13, AQ14 | |
| C02 | ⬜ | Command sender/receiver respecting pipeline ≤10 and delays | M | C01, S06 | RQ11, RQ12 | |
| C03 | ⬜ | Survival AI: move, see, pick food, avoid death | M | C02, S10 | RQ07, RQ20, AQ12, AQ13, AQ23 | |
| C04 | ⬜ | Gathering + broadcast meetup + enchantment attempts | L | C03, S12, S14 | RQ09, RQ15, RQ20, AQ25 | |
| C05 | ⬜ | Fork strategy when family needs slots | S | C03, S13 | RQ13, AQ26 | |

### Sprint 4 — Graphic client

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| G01 | 🟡 | GUI↔server connect path + render empty/live map | L | S04, A01 | RQ21, AQ15 | |
| G02 | ⬜ | Icons: players, food, all stone types visible | M | G01 | RQ21, AQ18, AQ28 | |
| G03 | ⬜ | Click square → floating details with counts | M | G02 | RQ21, AQ16, AQ17 | |
| G04 | ⬜ | Click player → characteristics overlay | M | G02 | RQ21, AQ19 | |
| G05 | ⬜ | Broadcast/sound visualization | M | G01, S12 | RQ21, AQ20 | |

### Sprint 5 — Integration

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| I01 | ⬜ | Scripted multi-client localhost smoke (server+2 AI) | M | C04, C05, S15 | RQ01, RQ02, RQ20 | |
| I02 | ⬜ | Walk all non-bonus AQs; record evidence commands in handoff note | M | I01, G05, S16 | _(all AQ01–AQ33 verification)_ | |

### Bonus (optional; after I02)

| ID | Status | Ticket | Size | Deps | Coverage | Claimed by |
|----|--------|--------|------|------|----------|------------|
| B01 | ⬜ | 3D or alternate visual representation | L | G05 | RQB1, AQB1 | |
| B02 | ⬜ | Server `--log` / log-mode flag | S | S01 | RQB2, AQB2 | |
| B03 | ⬜ | Resource/food density CLI flag | S | S04 | RQB3, AQB3 | |
| B04 | ⬜ | Seed flag server (+ client if needed) for reproducibility | M | S04, C01 | RQB4, AQB4 | |

---

## 7. Verification gates

| Gate | When | Pass criteria | Evidence |
|------|------|---------------|----------|
| G0 | End Sprint 0 | `scripts/check.sh` (or documented cmds) runs; stubs print usage | Local terminal output |
| G1 | After S06 | Handshake + queue + time skeleton tests green | `cargo test` |
| G2 | After S16 | Server AQs for bind/welcome/exec/timings/food/vision/torus addressable | Manual + tests |
| G3 | After C05 | AI client AQs + autonomous loop on localhost | Run logs |
| G4 | After G05 | GUI AQs (map, click, sounds, entities) | Manual checklist |
| G5 | After I02 | Full non-bonus AQ pass dry-run | Handoff note listing AQ01–AQ33 |
| GB | After bonuses | AQB* only if claimed | Demo flags / screenshots |

---

## 8. Requirements coverage matrix

| RQ | Tickets | Gate |
|----|---------|------|
| RQ01 | A01, I01 | G0, G5 |
| RQ02 | S15, I01 | G2, G5 |
| RQ03 | S04, S07 | G2 |
| RQ04 | S04 | G2 |
| RQ05 | S04 | G2 |
| RQ06 | S05 | G2 |
| RQ07 | S10, C03 | G2, G3 |
| RQ08 | S08 | G2 |
| RQ09 | S14, C04 | G2, G3 |
| RQ10 | S06 | G1 |
| RQ11 | S06, S07, S09, C02 | G1–G3 |
| RQ12 | S06, C02 | G1, G3 |
| RQ13 | S13, C05 | G2, G3 |
| RQ14 | S11 | G2 |
| RQ15 | S12, C04 | G2, G3 |
| RQ16 | A04, S02, S16 | G0, G2 |
| RQ17 | A02, S01 | G0, G1 |
| RQ18 | A02, C01 | G0, G3 |
| RQ19 | S03, C01 | G1, G3 |
| RQ20 | C03, C04, I01 | G3, G5 |
| RQ21 | G01–G05 | G4 |
| RQB1 | B01 | GB |
| RQB2 | B02 | GB |
| RQB3 | B03 | GB |
| RQB4 | B04 | GB |

## 9. Audit coverage matrix

| AQ | Tickets | Gate |
|----|---------|------|
| AQ01 | A02, S01 | G0, G1 |
| AQ02 | S02 | G1 |
| AQ03 | S16 | G2 |
| AQ04 | S16 | G2 |
| AQ05 | S16 | G2 |
| AQ06 | S06, S07 | G1, G2 |
| AQ07 | S10 | G2 |
| AQ08 | S10 | G2 |
| AQ09 | S08 | G2 |
| AQ10 | S04, S07 | G2 |
| AQ11 | A02, C01 | G0, G3 |
| AQ12 | C01, C03 | G3 |
| AQ13 | C01, C03 | G3 |
| AQ14 | S03, C01 | G1, G3 |
| AQ15 | G01 | G4 |
| AQ16 | G03 | G4 |
| AQ17 | G03 | G4 |
| AQ18 | G02 | G4 |
| AQ19 | G04 | G4 |
| AQ20 | G05 | G4 |
| AQ21 | S05 | G2 |
| AQ22 | S05 | G2 |
| AQ23 | S09, C03 | G2, G3 |
| AQ24 | S09 | G2 |
| AQ25 | S14, C04 | G2, G3 |
| AQ26 | S13, C05 | G2, G3 |
| AQ27 | S04 | G2 |
| AQ28 | S04, G02 | G2, G4 |
| AQ29 | S04 | G2 |
| AQ30 | S10 | G2 |
| AQ31 | S14 | G2 |
| AQ32 | S12 | G2 |
| AQ33 | S12 | G2 |
| AQB1 | B01 | GB |
| AQB2 | B02 | GB |
| AQB3 | B03 | GB |
| AQB4 | B04 | GB |

**Orphan check:** every RQ01–RQ21, RQB1–4, AQ01–AQ33, AQB1–4 appears above ≥1 time.

---

## 10. Immediate next work queue

1. ~~**S01**–**S11**~~ ✅ done (through kick)
2. **S12** — broadcast (🟡 Ready; Deps S07 ✅)
3. **S13** — fork / connect_nbr (🟡 Ready; Deps S05+S06 ✅)
4. **S14** — enchantment ritual (🟡 Ready; Deps S09+S10 ✅)

## 11. Summary by track

| Track | Total | ✅ | 🟢 | 🟡/⬜ | 🔴 |
|-------|------:|--:|--:|-----:|--:|
| A Bootstrap | 4 | 4 | 0 | 0 | 0 |
| S Server | 16 | 11 | 0 | 5 | 0 |
| C Client | 5 | 0 | 0 | 5 | 0 |
| G Graphic | 5 | 0 | 0 | 5 | 0 |
| I Integration | 2 | 0 | 0 | 2 | 0 |
| B Bonus | 4 | 0 | 0 | 4 | 0 |
| **All** | **36** | **15** | **0** | **21** | **0** |

Core (non-bonus) tickets: **32**. Bonus: **4**.

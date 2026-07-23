# AGENTS.md — Zappy

Permanent coding-agent instructions for this repository.

## 1. Project overview

**Zappy** is an entirely automatic multiplayer game: AI clients connect to a shared TCP server, form teams/families, survive on food, collect stones, and perform evolution rituals to level up. The game ends when **six members of one team reach level 8**.

| It is | It is not |
|-------|-----------|
| A localhost TCP simulation (server + AI clients + graphic client) | A hosted multiplayer service |
| Driven by autonomous AI clients (no human piloting during play) | A human-vs-human action game |
| Audited against immutable specs in `docs/raw/` | Free to reinterpret win rules or protocol syntax |
| Built turn-by-turn on `main` | A PR-based / parallel-merge workflow |

## 2. Source of Truth

| Document | Path | Purpose |
|----------|------|---------|
| Requirements | `docs/raw/requirements.md` | What must be built (**READ-ONLY**) |
| Audit Gate | `docs/raw/audit.md` | Pass/fail acceptance (**READ-ONLY**) |
| Spec (product) | `docs/PRD.md` | Goals, users, features, non-goals |
| Spec (technical) | `docs/SDS.md` | APIs, modules, data model, contracts |
| Tracker | `docs/ticket-tracker.md` | Tickets, status, handoff, coverage |
| This file | `AGENTS.md` | Agent coding guidelines |
| Workflow | `docs/AGENT_WORKFLOW.md` | Paste-ready “implement one ticket” prompt |

**Priority when docs disagree:** `docs/raw/requirements.md` → `docs/raw/audit.md` → `docs/PRD.md` / `docs/SDS.md` → tracker notes.

RQ/AQ ID legends live in `docs/ticket-tracker.md` (raw files are unnumbered).

## 3. Technology constraints

| Area | Allowed | Chosen / required | Forbidden |
|------|---------|-------------------|-----------|
| Server language | C, C++, Rust | **Rust** (edition 2021+) | Other languages for the server binary |
| Server I/O | Multiplexed TCP (e.g. `mio` / `tokio`) | Non-blocking; never hang forever | Blocking-per-connection architectures that freeze the world; `exec*` family |
| AI client | Any language | **Python 3.11+** | Human input during play; out-of-band client↔client IPC |
| Graphic client | JS/TS, Python, C/C++, etc. | **TypeScript + HTML5 Canvas** (browser or local static server) | **Any game engine** (Unity, Godot, Unreal, Phaser as engine, etc.) |
| Tests | Language-native | `cargo test` (server), `pytest` (client), Playwright or manual checklist (GUI) | Skipping gates before ✅ |
| Lint / format | Stack-native | `cargo fmt` + `clippy`, `ruff`, `eslint`/`prettier` | Ignoring lint in Done tickets |
| Platform | Linux / WSL2 localhost | Primary target | Assuming remote hosting |
| Stress tools | `siege` **only** against own server | Optional local stress | Siege / DoS against anything you do not own |

## 4. Directory structure

```text
zappy/
├── AGENTS.md                 # This file
├── README.md                 # Human quickstart (created in Sprint 0)
├── docs/
│   ├── raw/                  # IMMUTABLE acceptance sources
│   │   ├── requirements.md
│   │   └── audit.md
│   ├── PRD.md
│   ├── SDS.md
│   ├── ticket-tracker.md
│   ├── AGENT_WORKFLOW.md
│   └── handoff-notes/        # Optional short turn summaries
├── server/                   # Rust TCP game server → ./server
├── client/                   # Python autonomous AI → ./client
├── gui/                      # TypeScript + Canvas graphic client
└── scripts/                  # Dev helpers (build, lint, test)
```

`docs/raw/` is **immutable** once agreed. Do not edit it unless a human explicitly asks.

## 5. Coding standards

### Server (Rust)
- Public binary name / wrapper: `./server` matching CLI in requirements.
- Prefer clear modules: `cli`, `net`, `world`, `player`, `commands`, `time`, `protocol`.
- No `std::process::Command` / `exec`-equivalent to spawn another server.
- Command queue per player; max **10** pending requests; unknown commands → `ko\n`.
- All action delays use time unit `1/t` seconds (`t` default **100**).

### AI client (Python)
- Entry point usable as `./client` (script or thin wrapper).
- Fully autonomous after connect; no stdin gameplay.
- Protocol strings must match the requirements tables exactly (lowercase commands, `\n` terminated).

### Graphic client (TypeScript + Canvas)
- At least 2D with icons for map, players, food, stones.
- Click square → detail overlay (tooltip / floating panel).
- Click player → characteristics overlay.
- Visualize broadcasts/sounds.
- No game engines.

### General
- Prefer small, testable units over large god-objects.
- Do not invent protocol fields or rename stone types.
- Document resource-generation rules in code comments **and** `docs/SDS.md` (auditors will ask).

## 6. Critical behavior checklist (main deliverable)

Before calling server/gameplay work done, confirm:

- [ ] `./server` without args prints usage (port, width, height, teams, `-c`, `-t`)
- [ ] TCP clients receive `WELCOME\n`; handshake returns `nb-client` then `X Y`
- [ ] Unknown team → server prints `Error: the team <name> doesn't exist`; client disconnected
- [ ] World is toroidal; resources obey documented generation rules
- [ ] Players start level 1, 10 food (1260 time units), 0 stones
- [ ] Food depletes; starvation → `death`; eating extends life (126 TU / food)
- [ ] Commands & delays match the requirements table; buffer ≤ 10
- [ ] Vision grows with level; `see` format matches subject
- [ ] Ritual stone/player table exact; win = 6 teammates at level 8
- [ ] `broadcast` → `message <K>,<text>` with correct sound direction
- [ ] No `exec*`; server survives local stress; second bind fails cleanly
- [ ] AI client runs unaided; GUI shows map, entities, square/player details, sounds

## 7. Testing guidelines

| Layer | How |
|-------|-----|
| Server unit | `cargo test` in `server/` (map wrap, vision tiles, ritual table, sound K, resource rules) |
| Server protocol | Integration tests or scripted TCP clients (handshake, commands, timings at high `t`) |
| AI client | `pytest` in `client/` (CLI parsing, protocol helpers; optional mock server) |
| GUI | Manual audit checklist + optional smoke script; click square/player, sound viz |
| Audit gate | Walk `docs/raw/audit.md` questions mapped to AQ IDs in the tracker |

**Rule:** verify listed AQ/RQ IDs against `docs/raw/audit.md` before marking a ticket ✅.

## 8. Dev workflow commands

After Sprint 0 scaffolding exists (exact scripts may live under `scripts/`):

```bash
git pull origin main

# Server
cd server && cargo build --release && cargo test && cargo fmt --check && cargo clippy -- -D warnings

# AI client
cd client && ruff check . && pytest

# GUI
cd gui && npm test && npm run lint   # or project-equivalent

# Manual binaries (wrappers expected at repo root or documented paths)
./server -p 8080 -x 10 -y 10 -c 5 -n team1 team2 -t 100
./client -n team1 -p 8080 -h 127.0.0.1
```

If a command is not wired yet, the open Sprint 0 / early server tickets own creating it — do not invent alternate CLIs that break the subject.

## 9. Collaboration model (turn-based, no PRs)

- **Branch:** work on **`main` only** (prefer direct pushes for this project size).
- **One active worker at a time** (one human or one agent session).
- **Before starting:**
  1. `git pull` latest `main`
  2. Read `docs/ticket-tracker.md`
  3. Claim **exactly one** 🟡 Ready / ⬜ Not Started ticket whose Deps are all ✅
  4. Set status → 🟢 and set **Claimed by**
- **While working:** implement only that ticket’s scope; do not start the next ticket.
- **After finishing:**
  1. Tests / lint for touched areas pass
  2. Covered RQ/AQ IDs verified against raw audit/requirements
  3. Tracker → ✅ (Claimed by may remain as history)
  4. Optional: `docs/handoff-notes/{ID}-{slug}.md`
  5. **Push to `main`**
- **Never leave the tracker stale** — status is the handoff signal.
- Update “Who’s up / last push” in the tracker when you push.

## 10. Done checklist

Replace any PR checklist with this:

- [ ] Code runs for the ticket’s deliverable
- [ ] Tests pass
- [ ] Lint passes
- [ ] Covered RQ/AQ IDs verified
- [ ] Tracker updated to ✅
- [ ] Pushed to `main`

## 11. Audit-before-Done rule

Before setting a ticket to ✅, re-read the AQ items in its Coverage column and confirm they would pass a yes/no check from `docs/raw/audit.md`. If evidence is missing, the ticket is not done.

## 12. Immutable raw docs rule

**Do not edit** `docs/raw/requirements.md` or `docs/raw/audit.md` unless a human explicitly asks. Specs and tickets may *reference* them; they do not replace them.

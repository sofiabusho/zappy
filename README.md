# Zappy

Entirely automatic multiplayer simulation: AI clients connect to a shared TCP server, form teams, survive, collect stones, and evolve. A graphic client visualizes the world.

**Platform:** Linux / WSL2 **localhost only**. This is not a hosted multiplayer service.

## Parts (RQ01)

| Directory   | Role                                            |
| ----------- | ----------------------------------------------- |
| `server/`   | Rust TCP game server (`./server/server`)        |
| `client/`   | Python autonomous AI client (`./client/client`) |
| `gui/`      | TypeScript + HTML5 Canvas graphic client        |
| `scripts/`  | Dev helpers (build, lint, test)                 |

## Prerequisites

| Tool | Notes |
|------|--------|
| Rust (edition 2021+) | `cargo`, `rustfmt`, `clippy` |
| Python 3.11+ | AI client; `ruff` / `pytest` (or `./scripts/ensure-dev-tools.sh`) |
| Node.js 18+ / npm | Graphic client lint/build |
| Optional: `telnet`, `siege` | Local audit only (see warning below) |

## Build & check

```bash
git pull origin main
(cd gui && npm install)    # one-time / when package.json changes
./scripts/check.sh         # cargo fmt/clippy/test + ruff/pytest + GUI lint + script tests
```

Per-stack commands (same gates as `check.sh`):

```bash
# Server
cd server && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

# AI client
cd client && ruff check . && pytest   # or: python3 -m pytest

# GUI
cd gui && npm run lint && npm test
```

## Run (Sprint 0 stubs)

Entrypoints are documented paths (a root file named `./server` cannot coexist with `server/` on Unix):

```bash
./server/server    # usage stub (AQ01); real CLI → S01
./client/client    # usage stub (AQ11); real CLI → C01
```

When the real server/client exist, the subject-shaped commands look like:

```bash
./server/server -p 8080 -x 10 -y 10 -c 5 -n team1 team2 -t 100
./client/client -n team1 -p 8080 -h 127.0.0.1
```

Default client host is **localhost** / `127.0.0.1`.

## Audit quickstart (localhost)

Acceptance sources (read-only): [`docs/raw/requirements.md`](./docs/raw/requirements.md), [`docs/raw/audit.md`](./docs/raw/audit.md). RQ/AQ IDs live in [`docs/ticket-tracker.md`](./docs/ticket-tracker.md).

Typical local checks once the server is implemented (S-track):

```bash
# Usage
./server/server

# Welcome handshake (after S02)
./server/server -p 8080 -x 10 -y 10 -c 5 -n my_team -t 10
# other terminal:
telnet 127.0.0.1 8080

# Bind conflict (AQ05): second server on same port should fail cleanly
# (e.g. "Address already in use")
```

### Siege / stress (own server only)

```bash
# ONLY against your own local Zappy server, e.g.:
siege -b 127.0.0.1:8080
```

> **Warning:** `siege` is a stressing tool. Use it **ONLY** to test **your own** server on localhost. Do **NEVER** use it on any server/website without the owner's permission — that is illegal DDoS and can have serious consequences. (See subject note in `docs/raw/requirements.md`.)

## Server constraints (RQ16)

Documented for auditors and implementers (enforced in code by later S-track tickets, especially S02/S16):

| Constraint | Project choice / rule |
|------------|------------------------|
| Language | **Rust** (allowed: C, C++, Rust) |
| I/O | Multiplexed TCP; non-blocking event loop |
| Availability | Requests must **never hang forever**; server stays available |
| Forbidden | No `exec*` family to spawn another server |
| Deployment | Localhost only — no need to host remotely |

## Agent / contributor guide

**Start here:** [`AGENTS.md`](./AGENTS.md) — source-of-truth order, stack constraints, turn-based workflow, and done checklist.

Also useful:

- [`docs/ticket-tracker.md`](./docs/ticket-tracker.md) — tickets and coverage
- [`docs/PRD.md`](./docs/PRD.md) / [`docs/SDS.md`](./docs/SDS.md) — product & technical specs
- [`docs/AGENT_WORKFLOW.md`](./docs/AGENT_WORKFLOW.md) — paste-ready one-ticket prompt
- [`docs/raw/`](./docs/raw/) — immutable acceptance (do not edit)

# Zappy

Entirely automatic multiplayer simulation: AI clients connect to a shared TCP server, form teams, survive, collect stones, and evolve. A graphic client visualizes the world.

## Parts (RQ01)

| Directory   | Role                                      |
| ----------- | ----------------------------------------- |
| `server/`   | Rust TCP game server (`./server`)         |
| `client/`   | Python autonomous AI client (`./client`)  |
| `gui/`      | TypeScript + HTML5 Canvas graphic client  |
| `scripts/`  | Dev helpers (build, lint, test)           |

## Agent / contributor guide

**Start here:** [`AGENTS.md`](./AGENTS.md) — source-of-truth order, stack constraints, turn-based workflow, and done checklist.

Also useful:

- [`docs/ticket-tracker.md`](./docs/ticket-tracker.md) — tickets and coverage
- [`docs/PRD.md`](./docs/PRD.md) / [`docs/SDS.md`](./docs/SDS.md) — product & technical specs
- [`docs/raw/requirements.md`](./docs/raw/requirements.md) / [`docs/raw/audit.md`](./docs/raw/audit.md) — immutable acceptance (read-only)

## Localhost only

This project targets Linux / WSL2 localhost. Do not stress-test or siege hosts you do not own.

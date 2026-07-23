# PRD — Zappy

Product requirements derived from `docs/raw/requirements.md`. Raw docs remain authoritative.

## Goals

- Ship a complete **localhost** Zappy stack: game **server**, autonomous **AI client**, and **graphic client**.
- AIs play without human intervention after launch; teams compete to get **six players to level 8**.
- Satisfy functional audit questions in `docs/raw/audit.md` (AQ IDs in the tracker).

## Users

| User | Need |
|------|------|
| Peer auditor | Run CLI binaries, telnet/siege checks, watch GUI, ask about resource rules |
| Developer / agent | Implement one ticket at a time against clear protocol and timing rules |
| Spectator (optional) | Watch the world via the graphic client |

## Primary features

1. **World server** — Toroidal plains map; food + six stone types; multiplexed TCP; timed command execution; win detection (RQ/AQ: server + world + protocol clusters).
2. **Player lifecycle** — Start loadout, hunger/death, inventory, vision by level, kick, fork/ship, connect_nbr.
3. **Evolution ritual** — Exact player/stone table from requirements; enchantment flow and failure-when-alone rule.
4. **Communication** — In-game `broadcast` only; directional `message <K>,<text>`; no out-of-band client chat.
5. **Autonomous AI client** — Connects, survives, gathers, coordinates via broadcast, forks as needed.
6. **Graphic client** — Real-time 2D map; entities visible; square + player detail overlays; sound visualization.

## Non-goals

- Hosting / public internet deployment.
- Human-controlled gameplay clients as the primary deliverable.
- Game engines for the visualizer.
- Changing stone names, ritual table, or command syntax from raw requirements.
- Parallel PR workflows (this repo uses turn-based `main`).

## Success metrics

| Metric | Signal |
|--------|--------|
| Audit readiness | All non-bonus AQs answerable “yes” with local evidence |
| Protocol fidelity | Handshake, commands, delays, and responses match raw tables |
| Autonomy | AI clients run a game without keyboard input |
| Visualization | Auditor can see map, resources, players, square details, sounds |
| Process health | Tracker always reflects who owns the current turn |

## Bonus (optional)

- 3D (or alternate) visualizer
- Server log mode flag
- Resource/food density flag
- Reproducible seed flag (server ± client)

Marked optional in tracker Coverage as RQB*/AQB*.

## Assumptions

See `docs/ticket-tracker.md` → Assumptions. Stack choice (Rust / Python / TS+Canvas) is a project decision within allowed languages.

# Handoff note — `A01` `repo-scaffold`

> Short turn summary. Not a PR.

## Summary

- Created `server/`, `client/`, `gui/`, `scripts/` with per-dir READMEs (RQ01 parts).
- Root `README.md` points at `AGENTS.md` and documents the three game parts.
- Added `scripts/test_scaffold.py` to assert dirs + README link; basic `.gitignore` for Python caches.

## Files touched

- `README.md`
- `server/README.md`, `client/README.md`, `gui/README.md`, `scripts/README.md`
- `scripts/test_scaffold.py`
- `.gitignore`
- `docs/ticket-tracker.md`
- `docs/handoff-notes/A01-repo-scaffold.md`

## How to verify

```bash
python3 scripts/test_scaffold.py
python3 -m py_compile scripts/test_scaffold.py
ls server client gui scripts README.md AGENTS.md
```

- AQ / RQ IDs checked: **RQ01** — requirements “The Game Parts” (server, clients, graphic client) mapped to `server/`, `client/`, `gui/` in tree + README.

## Risks / follow-ups

- No `./server` / `./client` wrappers yet (A02).
- Full lint/test harness (`scripts/check.sh`, cargo/ruff/eslint) is A03.

## Next suggested ticket

- `A02` — Add `./server` and `./client` wrapper stubs + placeholder usage strings matching subject

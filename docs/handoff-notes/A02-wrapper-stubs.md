# Handoff note — `A02` `wrapper-stubs`

> Short turn summary. Not a PR.

## Summary

- Added executable wrapper stubs `server/server` and `client/client` that print audit-matching usage and exit 1.
- Stubs delegate to `server/stub.sh` and `client/stub.py`.
- Documented entrypoints in root README (Unix: root `./server` cannot coexist with `server/` dir; AGENTS allows documented paths).
- Added `scripts/test_wrappers.py` for AQ01/AQ11/RQ17/RQ18 checks.

## Files touched

- `server/server`, `server/stub.sh`, `server/README.md`
- `client/client`, `client/stub.py`, `client/README.md`
- `scripts/test_wrappers.py`, `scripts/README.md`
- `README.md`
- `docs/ticket-tracker.md`
- `docs/handoff-notes/A02-wrapper-stubs.md`

## How to verify

```bash
./server/server
./client/client
python3 scripts/test_wrappers.py
python3 -m py_compile scripts/test_wrappers.py client/stub.py
```

- AQ / RQ IDs checked:
  - **RQ17** — usage documents `-p -x -y -n -c [-t]` (audit wording)
  - **RQ18** — usage documents `-n -p [-h]` and default localhost
  - **AQ01** — server stub prints usage like `docs/raw/audit.md` sample
  - **AQ11** — client stub prints usage like `docs/raw/audit.md` sample

## Risks / follow-ups

- Entrypoints are `./server/server` and `./client/client`, not root `./server` / `./client` (filesystem name conflict with package dirs). A04 may further clarify for auditors.
- Stubs always print usage (no real arg parse yet) — S01 / C01 own real CLI.

## Next suggested ticket

- `A03` — Wire lint/format/test commands + `scripts/check.sh`

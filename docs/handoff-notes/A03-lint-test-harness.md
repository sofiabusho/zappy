# Handoff note — `A03` `lint-test-harness`

> Short turn summary. Not a PR.

## Summary

- Wired stack-native lint/format/test for all three parts + `scripts/check.sh` (G0).
- Server: minimal `zappy_server` Rust lib with `cargo fmt` / `clippy` / `test`.
- Client: `pyproject.toml`, `requirements-dev.txt`, pytest for stub usage, `ruff check`.
- GUI: TypeScript scaffold with `eslint` + `tsc --noEmit` via `npm run lint` / `npm test`.
- `scripts/ensure-dev-tools.sh` bootstraps ruff + pytest into gitignored `.tools/` when missing.

## Files touched

- `server/Cargo.toml`, `server/Cargo.lock`, `server/src/lib.rs`, `server/README.md`
- `client/pyproject.toml`, `client/requirements-dev.txt`, `client/tests/test_stub_usage.py`, `client/README.md`
- `gui/package.json`, `gui/package-lock.json`, `gui/tsconfig.json`, `gui/eslint.config.mjs`, `gui/src/scaffold.ts`, `gui/README.md`
- `scripts/check.sh`, `scripts/ensure-dev-tools.sh`, `scripts/test_check_harness.py`, `scripts/README.md`
- `.gitignore`, `README.md`
- `docs/ticket-tracker.md`, `docs/handoff-notes/A03-lint-test-harness.md`

## How to verify

```bash
(cd gui && npm install)   # if needed
./scripts/check.sh
```

- AQ / RQ IDs checked: **none** (ticket Coverage is `—`). Harness supports G0 process gate.

## Risks / follow-ups

- No system `pip` in this environment; `.tools/` bootstrap covers ruff/pytest. Prefer `pip install -r client/requirements-dev.txt` when available.
- GUI eslint packages warn on Node 18 engines but lint/test currently pass.
- A04 should expand README audit/siege guidance (RQ16).

## Next suggested ticket

- `A04` — Document build/run/audit quickstart in README (localhost only; siege warning)

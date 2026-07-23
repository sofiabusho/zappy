# scripts/

Dev helpers for build, lint, and test.

## Full gate (A03 / G0)

```bash
# GUI deps (one-time / when package.json changes)
(cd gui && npm install)

# Optional: system Python tooling instead of local .tools bootstrap
# pip3 install -r client/requirements-dev.txt

./scripts/check.sh
```

`check.sh` runs:

| Area | Commands |
|------|----------|
| bootstrap | `ensure-dev-tools.sh` (fetches ruff/pytest into `.tools/` if missing) |
| server | `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` |
| client | `ruff check .`, `pytest` |
| gui | `npm run lint`, `npm test` |
| scripts | `test_scaffold.py`, `test_wrappers.py`, `test_check_harness.py`, `test_readme_quickstart.py` |

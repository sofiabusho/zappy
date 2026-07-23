# client/

Python autonomous AI client for Zappy.

Entrypoint (stub): `./client/client` — prints subject usage until C01 implements real CLI.

## Lint / test

```bash
pip3 install -r requirements-dev.txt
ruff check .
pytest
```

Or from repo root: `./scripts/check.sh`

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md).

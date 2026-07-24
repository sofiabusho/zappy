# client/

Python autonomous AI client for Zappy.

Entrypoint: `./client/client` (the root `./client` name is taken by this
directory on Unix). Wraps `python3 -m zappy_client`.

```bash
./client/client -n <team> -p <port> [-h <hostname>]   # default host: localhost
```

C01 implements the real CLI, TCP connect, and the subject handshake
(`WELCOME` → team → `nb-client` → `x y`). An unknown/full team is reported and
the client exits non-zero (the server prints `Error: the team <name> doesn't
exist`). Autonomous gameplay (survival, ritual, fork) lands in C02+.

## Layout

| Path | Purpose |
|------|---------|
| `zappy_client/cli.py` | Argument parsing + subject usage string |
| `zappy_client/protocol.py` | Line-buffered TCP connection + handshake |
| `zappy_client/main.py` | Entrypoint: connect → handshake → idle reader |

## Lint / test

```bash
pip3 install -r requirements-dev.txt
ruff check .
ruff format --check .
pytest
```

Or from repo root: `./scripts/check.sh`

See [`AGENTS.md`](../AGENTS.md) and [`docs/SDS.md`](../docs/SDS.md).

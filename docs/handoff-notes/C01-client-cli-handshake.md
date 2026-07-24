# Handoff note — `C01` `client-cli-handshake`

> Short turn summary. Not a PR.

## Summary

- Replaced the A02 client stub with a real `zappy_client` Python package:
  `cli.py` (subject `-n -p [-h]` parser + verbatim usage), `protocol.py`
  (line-buffered TCP `ServerConnection` + `handshake`), `main.py` (entrypoint).
- Handshake mirrors `server/src/net.rs`: read `WELCOME`, send team, read
  `nb-client`, read `x y` (RQ19). Default host is `localhost` (RQ18).
- Unknown/full team → server closes after the team line; client raises
  `HandshakeError`, prints a message, exits non-zero (AQ14). Server still
  prints `Error: the team <name> doesn't exist` on its side.
- After a successful handshake the client idles (echoing server pushes such as
  `message`/`death`) and exits cleanly on close — autonomous, no stdin.
  Gameplay (survival/ritual/fork) is deliberately deferred to C02+.
- `client/client` wrapper now runs `python3 -m zappy_client` (no `exec*`);
  removed obsolete `client/stub.py` + `test_stub_usage.py`.

## Files touched

- `client/zappy_client/{__init__,__main__,cli,protocol,main}.py` (new package)
- `client/client` (wrapper → real module), `client/README.md`
- `client/tests/test_cli.py`, `client/tests/test_handshake.py` (new)
- removed `client/stub.py`, `client/tests/test_stub_usage.py`
- `docs/ticket-tracker.md` (C01 ✅; who's-up; counts)
- `docs/handoff-notes/C01-client-cli-handshake.md` (this file)

## How to verify

```bash
cd client && ruff check . && ruff format --check . && pytest   # 21 passed

# Live smoke (from repo root):
./server/server -p 8199 -x 12 -y 8 -c 5 -n my_team -t 100 &
./client/client                                   # AQ11: prints usage, exit 1
./client/client -n my_team -p 8199 -h 127.0.0.1   # AQ12/13: "joined team 'my_team' ... world 12x8, 4 free slot(s)."
./client/client -n wrong_team -p 8199 -h 127.0.0.1 # AQ14: handshake failed; server prints team error
```

- AQ / RQ IDs checked: **RQ18** (`-n -p [-h]`, default localhost, autonomous),
  **RQ19** (WELCOME/team/nb-client/x y; bad team errors), **AQ11** (usage),
  **AQ12** (launches vs server, no errors), **AQ13** (`-h 127.0.0.1` clean),
  **AQ14** (wrong team → server error + client kicked). All confirmed via the
  live smoke above against the real Rust server.

## Risks / follow-ups

- Client sends no commands yet; an idle player eventually starves (`death`) and
  the server drops it — expected until C02/C03 add the command loop + survival.
- Pipeline cap (≤10) and per-command delays are enforced server-side; the
  client-side pipeline discipline is C02's scope.

## Next suggested ticket

- `C02` — command sender/receiver respecting pipeline ≤10 and delays
  (Deps C01 ✅, S06 ✅), or `G01` GUI scaffold.

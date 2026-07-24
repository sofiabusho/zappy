"""``./client`` entrypoint (C01 / RQ18, RQ19, AQ11-AQ14).

Responsibilities for this ticket:
  * parse the subject CLI (``-n -p [-h]``), printing usage on error (AQ11);
  * connect over TCP to the server (AQ12/AQ13);
  * complete the handshake and report the world size / free slots (RQ19);
  * exit cleanly if the team is rejected (AQ14) or the server closes.

Autonomous gameplay (survival, ritual, fork) is intentionally out of scope
here and lands in C02+. After a successful handshake this client keeps the
connection open and echoes any server-initiated lines (e.g. ``message`` or
``death``) so an auditor can watch it interact, exiting when the server closes.
"""

from __future__ import annotations

import sys

from .cli import USAGE, ClientArgs, UsageError, parse_args
from .protocol import (
    ConnectionClosed,
    HandshakeError,
    HandshakeResult,
    ServerConnection,
    handshake,
)


def _idle(conn: ServerConnection) -> int:
    """Read server-initiated lines until the connection closes.

    Keeps the socket alive so the client "interacts without errors" (AQ12/13)
    without driving gameplay (that is C02+). Returns 0 on a clean close.
    """
    conn._sock.settimeout(None)  # block for pushed lines; no busy-loop
    while True:
        try:
            line = conn.recv_line()
        except ConnectionClosed:
            return 0
        if line == "death":
            print("client: player died (starvation); exiting.")
            return 0
        print(f"server> {line}")


def run(args: ClientArgs) -> int:
    """Connect, handshake, and hand off to the idle reader."""
    try:
        conn = ServerConnection.connect(args.host, args.port)
    except OSError as exc:
        print(f"client: could not connect to {args.host}:{args.port}: {exc}", file=sys.stderr)
        return 1

    with conn:
        try:
            result: HandshakeResult = handshake(conn, args.team)
        except HandshakeError as exc:
            print(f"client: handshake failed: {exc}", file=sys.stderr)
            return 1
        except ConnectionClosed as exc:
            print(f"client: server closed during handshake: {exc}", file=sys.stderr)
            return 1

        print(
            f"client: joined team {args.team!r} on {args.host}:{args.port} "
            f"— world {result.width}x{result.height}, {result.nb_client} free slot(s)."
        )
        try:
            return _idle(conn)
        except KeyboardInterrupt:
            print("\nclient: interrupted; disconnecting.")
            return 0


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint. Prints usage and returns non-zero on argument errors."""
    raw = sys.argv[1:] if argv is None else argv
    try:
        args = parse_args(raw)
    except UsageError:
        sys.stdout.write(USAGE)
        return 1
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())

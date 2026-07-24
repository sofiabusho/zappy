"""``./client`` entrypoint (C01 handshake, C03 survival, C04 gather + evolve).

Responsibilities:
  * parse the subject CLI (``-n -p [-h]``), printing usage on error (AQ11);
  * connect over TCP to the server (AQ12/AQ13);
  * complete the handshake and report the world size / free slots (RQ19);
  * exit cleanly if the team is rejected (AQ14) or the server closes;
  * then hand the connection to the autonomous :class:`GatheringAgent`, which
    plays with no human intervention (RQ18/RQ20): it sees, moves, and eats to
    stay alive (RQ07 / AQ23), gathers ritual stones, rallies same-level partners
    with ``broadcast`` (RQ15), and performs the evolution ritual to level up
    (RQ09 / AQ25).

Forking for family slots builds on this in C05.
"""

from __future__ import annotations

import sys

from .cli import USAGE, ClientArgs, UsageError, parse_args
from .gathering import GatheringAgent
from .pipeline import CommandPipeline
from .protocol import (
    ConnectionClosed,
    HandshakeError,
    HandshakeResult,
    ServerConnection,
    handshake,
)

# Gameplay socket timeout: every subject action replies within ``cost/t`` seconds
# (≤7/t for the verbs the survival loop uses), so a generous ceiling keeps the
# client from hanging forever on an unresponsive server (RQ16 spirit) without
# tripping during normal play.
_GAMEPLAY_TIMEOUT_S = 30.0


def _play(conn: ServerConnection) -> int:
    """Run the autonomous gather/evolve loop until death or disconnect."""
    conn._sock.settimeout(_GAMEPLAY_TIMEOUT_S)
    agent = GatheringAgent(CommandPipeline(conn))
    return agent.run()


def run(args: ClientArgs) -> int:
    """Connect, handshake, and hand off to the autonomous survival agent."""
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
            return _play(conn)
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

"""TCP connection + subject handshake for the Zappy client (C01 / RQ19, AQ14).

The player protocol is line-oriented: every message ends with ``\\n``. This
module owns the socket, a small inbound line buffer, and the handshake state
machine that mirrors ``server/src/net.rs``:

======  ===========================================
S -> C  ``WELCOME``
C -> S  ``<team-name>``
S -> C  ``<nb-client>``          (free slots after this join)
S -> C  ``<width> <height>``
======  ===========================================

If the team is unknown (or the team is full), the server prints its error and
closes the socket **without** sending ``nb-client``. We surface that as
:class:`HandshakeError` so the entrypoint can exit cleanly (AQ14).
"""

from __future__ import annotations

import socket
from dataclasses import dataclass

WELCOME = "WELCOME"

# Cap a single protocol line so a misbehaving/hostile peer cannot make the
# client buffer unbounded memory while waiting for a newline.
_MAX_LINE_BYTES = 4096


class ConnectionClosed(Exception):
    """Raised when the peer closes the socket mid-line."""


class HandshakeError(Exception):
    """Raised when the handshake does not complete (bad team, full, malformed)."""


@dataclass(frozen=True)
class HandshakeResult:
    """Outcome of a successful handshake."""

    nb_client: int
    width: int
    height: int


class ServerConnection:
    """Line-buffered wrapper around a blocking TCP socket to the server."""

    def __init__(self, sock: socket.socket) -> None:
        self._sock = sock
        self._buf = bytearray()

    @classmethod
    def connect(cls, host: str, port: int, timeout: float | None = 10.0) -> ServerConnection:
        """Open a TCP connection to ``host:port`` (RQ18 default host handled by CLI)."""
        sock = socket.create_connection((host, port), timeout=timeout)
        # Handshake is request/response; keep a timeout so the client never
        # hangs forever if the server misbehaves, matching the server's own
        # "never block forever" contract (RQ16).
        sock.settimeout(timeout)
        return cls(sock)

    def recv_line(self) -> str:
        """Return the next ``\\n``-terminated line (newline stripped).

        Raises :class:`ConnectionClosed` if the peer closes before a full line
        arrives, or :class:`HandshakeError` if a single line exceeds the cap.
        """
        while True:
            nl = self._buf.find(b"\n")
            if nl != -1:
                line = self._buf[:nl]
                del self._buf[: nl + 1]
                if line.endswith(b"\r"):
                    line = line[:-1]
                return line.decode("utf-8", errors="replace")
            if len(self._buf) > _MAX_LINE_BYTES:
                raise HandshakeError("server line exceeded maximum length")
            chunk = self._sock.recv(1024)
            if not chunk:
                raise ConnectionClosed("server closed the connection")
            self._buf.extend(chunk)

    def send_line(self, text: str) -> None:
        """Send ``text`` followed by ``\\n`` (all protocol lines are newline-terminated)."""
        self._sock.sendall(f"{text}\n".encode())

    def close(self) -> None:
        try:
            self._sock.close()
        except OSError:
            pass

    def __enter__(self) -> ServerConnection:
        return self

    def __exit__(self, *_exc: object) -> None:
        self.close()


def handshake(conn: ServerConnection, team: str) -> HandshakeResult:
    """Run the subject handshake and return :class:`HandshakeResult`.

    Steps (RQ19):
      1. Read ``WELCOME``.
      2. Send ``team``.
      3. Read ``nb-client`` (free slots after this join).
      4. Read ``x y`` (world dimensions).

    A closed socket after step 2 means the team was rejected (unknown team or
    full team): the server printed ``Error: the team <name> doesn't exist`` and
    dropped us (AQ14). We raise :class:`HandshakeError` for the caller to report.
    """
    welcome = conn.recv_line()
    if welcome != WELCOME:
        raise HandshakeError(f"expected {WELCOME!r}, got {welcome!r}")

    conn.send_line(team)

    try:
        nb_line = conn.recv_line()
    except ConnectionClosed as exc:
        raise HandshakeError(
            f"server rejected team {team!r} (unknown or full) and closed the connection"
        ) from exc

    try:
        nb_client = int(nb_line.strip())
    except ValueError as exc:
        raise HandshakeError(f"expected nb-client integer, got {nb_line!r}") from exc

    dim_line = conn.recv_line()
    parts = dim_line.split()
    if len(parts) != 2:
        raise HandshakeError(f"expected '<x> <y>', got {dim_line!r}")
    try:
        width = int(parts[0])
        height = int(parts[1])
    except ValueError as exc:
        raise HandshakeError(f"expected integer dimensions, got {dim_line!r}") from exc

    return HandshakeResult(nb_client=nb_client, width=width, height=height)

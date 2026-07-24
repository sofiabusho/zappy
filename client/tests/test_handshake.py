"""Handshake tests against an in-process mock server (C01 / RQ19, AQ14).

A small threaded socket server mimics ``server/src/net.rs``:
  * sends ``WELCOME\\n``;
  * on a known team, replies ``<nb-client>\\n`` then ``<x> <y>\\n``;
  * on an unknown team, closes the socket without a handshake reply.
This keeps the tests fast and independent of the Rust build while still
exercising the exact byte-level protocol the real server speaks.
"""

from __future__ import annotations

import socket
import threading
from collections.abc import Iterator

import pytest

from zappy_client.protocol import (
    ConnectionClosed,
    HandshakeError,
    ServerConnection,
    handshake,
)

KNOWN_TEAM = "my_team"
NB_CLIENT = 4
WIDTH, HEIGHT = 10, 20


def _serve_one(sock: socket.socket) -> None:
    """Handle exactly one client with the subject handshake, then close."""
    try:
        conn, _addr = sock.accept()
    except OSError:
        return
    with conn:
        conn.sendall(b"WELCOME\n")
        buf = bytearray()
        while b"\n" not in buf:
            chunk = conn.recv(1024)
            if not chunk:
                return
            buf.extend(chunk)
        team = buf.split(b"\n", 1)[0].rstrip(b"\r").decode()
        if team == KNOWN_TEAM:
            conn.sendall(f"{NB_CLIENT}\n{WIDTH} {HEIGHT}\n".encode())
            # Hold the socket briefly so the client can read both lines.
            try:
                conn.recv(1)
            except OSError:
                pass
        # Unknown team: just fall out of the `with` and close (server would
        # have printed the error to its own stderr).


@pytest.fixture()
def mock_server() -> Iterator[int]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    port = listener.getsockname()[1]
    thread = threading.Thread(target=_serve_one, args=(listener,), daemon=True)
    thread.start()
    try:
        yield port
    finally:
        listener.close()
        thread.join(timeout=1.0)


def test_handshake_known_team(mock_server: int) -> None:
    with ServerConnection.connect("127.0.0.1", mock_server, timeout=2.0) as conn:
        result = handshake(conn, KNOWN_TEAM)
    assert result.nb_client == NB_CLIENT
    assert (result.width, result.height) == (WIDTH, HEIGHT)


def test_handshake_unknown_team_raises(mock_server: int) -> None:
    with ServerConnection.connect("127.0.0.1", mock_server, timeout=2.0) as conn:
        with pytest.raises(HandshakeError):
            handshake(conn, "wrong_team")


def test_bad_welcome_raises() -> None:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    port = listener.getsockname()[1]

    def serve() -> None:
        conn, _ = listener.accept()
        with conn:
            conn.sendall(b"NOPE\n")

    thread = threading.Thread(target=serve, daemon=True)
    thread.start()
    try:
        with ServerConnection.connect("127.0.0.1", port, timeout=2.0) as conn:
            with pytest.raises(HandshakeError):
                handshake(conn, KNOWN_TEAM)
    finally:
        listener.close()
        thread.join(timeout=1.0)


def test_recv_line_strips_crlf() -> None:
    a, b = socket.socketpair()
    try:
        server_conn = ServerConnection(a)
        b.sendall(b"hello\r\nworld\n")
        assert server_conn.recv_line() == "hello"
        assert server_conn.recv_line() == "world"
    finally:
        a.close()
        b.close()


def test_recv_line_raises_on_close() -> None:
    a, b = socket.socketpair()
    server_conn = ServerConnection(a)
    b.close()
    with pytest.raises(ConnectionClosed):
        server_conn.recv_line()
    a.close()

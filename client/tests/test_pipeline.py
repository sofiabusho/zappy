"""Command pipeline tests (C02 / RQ11, RQ12).

These exercise the two subject invariants directly with an in-process socket
pair (no threaded server needed): exact command syntax on the wire (RQ11) and
the ≤10 outstanding-request window with correct response/push routing (RQ12).
"""

from __future__ import annotations

import socket

import pytest

from zappy_client.pipeline import (
    MAX_PENDING,
    Command,
    CommandPipeline,
    EventKind,
    PipelineFull,
    broadcast,
    classify,
    drop,
    pick,
)
from zappy_client.protocol import ServerConnection


def _pipe() -> tuple[CommandPipeline, socket.socket]:
    """Return a pipeline whose socket is wired to a peer we read/feed in-test."""
    client_sock, peer = socket.socketpair()
    conn = ServerConnection(client_sock)
    return CommandPipeline(conn), peer


def _read_all(sock: socket.socket) -> bytes:
    """Drain everything currently readable from ``sock`` without blocking."""
    sock.setblocking(False)
    chunks = bytearray()
    try:
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            chunks.extend(chunk)
    except BlockingIOError:
        pass
    return bytes(chunks)


# --- RQ11: exact command syntax ------------------------------------------------


def test_zero_arg_commands_match_subject_table() -> None:
    assert Command.ADVANCE == "advance"
    assert Command.RIGHT == "right"
    assert Command.LEFT == "left"
    assert Command.SEE == "see"
    assert Command.INVENTORY == "inventory"
    assert Command.KICK == "kick"
    assert Command.ENCHANTMENT == "enchantment"
    assert Command.FORK == "fork"
    assert Command.CONNECT_NBR == "connect_nbr"


def test_argument_builders_match_subject_syntax() -> None:
    assert pick("food") == "pick food"
    assert drop("amber") == "drop amber"
    assert broadcast("meet at 3,4") == "broadcast meet at 3,4"


def test_broadcast_rejects_embedded_newline() -> None:
    # A newline would split one broadcast into two malformed wire lines.
    with pytest.raises(ValueError):
        broadcast("line1\nline2")


def test_sent_command_is_newline_terminated_on_the_wire() -> None:
    pipe, peer = _pipe()
    try:
        pipe.send(Command.SEE)
        assert _read_all(peer) == b"see\n"
    finally:
        peer.close()


def test_argument_command_serialized_exactly() -> None:
    pipe, peer = _pipe()
    try:
        pipe.send(pick("peridot"))
        assert _read_all(peer) == b"pick peridot\n"
    finally:
        peer.close()


# --- RQ12: ≤10 pipeline window -------------------------------------------------


def test_window_caps_at_ten_outstanding() -> None:
    pipe, peer = _pipe()
    try:
        for _ in range(MAX_PENDING):
            assert pipe.try_send(Command.ADVANCE) is True
        assert pipe.pending == MAX_PENDING
        assert pipe.has_capacity is False
        # 11th is refused and not written to the wire.
        assert pipe.try_send(Command.ADVANCE) is False
        assert _read_all(peer) == b"advance\n" * MAX_PENDING
    finally:
        peer.close()


def test_send_raises_when_full() -> None:
    pipe, peer = _pipe()
    try:
        for _ in range(MAX_PENDING):
            pipe.send(Command.ADVANCE)
        with pytest.raises(PipelineFull):
            pipe.send(Command.ADVANCE)
    finally:
        peer.close()


def test_response_frees_a_slot() -> None:
    pipe, peer = _pipe()
    try:
        for _ in range(MAX_PENDING):
            pipe.send(Command.ADVANCE)
        assert pipe.has_capacity is False
        peer.sendall(b"ok\n")
        event = pipe.recv()
        assert event.kind is EventKind.RESPONSE
        assert event.text == "ok"
        assert event.command == "advance"
        assert pipe.pending == MAX_PENDING - 1
        # A freed slot lets one more through.
        assert pipe.try_send(Command.ADVANCE) is True
    finally:
        peer.close()


def test_responses_pair_fifo_with_commands() -> None:
    pipe, peer = _pipe()
    try:
        pipe.send(Command.INVENTORY)
        pipe.send(pick("food"))
        peer.sendall(b"{food 1260}\n")
        peer.sendall(b"ok\n")
        first = pipe.recv()
        second = pipe.recv()
        assert (first.command, first.text) == ("inventory", "{food 1260}")
        assert (second.command, second.text) == ("pick food", "ok")
        assert pipe.pending == 0
    finally:
        peer.close()


# --- push routing: pushes must not consume the response window -----------------


def test_classify_distinguishes_pushes_from_responses() -> None:
    assert classify("death") is EventKind.DEATH
    assert classify("message 3,hello") is EventKind.MESSAGE
    assert classify("moving 5") is EventKind.MOVING
    assert classify("ok") is EventKind.RESPONSE
    assert classify("ko") is EventKind.RESPONSE
    assert classify("{food 1260}") is EventKind.RESPONSE


def test_server_push_does_not_pop_a_pending_command() -> None:
    pipe, peer = _pipe()
    try:
        pipe.send(Command.SEE)
        # A broadcast heard mid-flight arrives before the `see` response.
        peer.sendall(b"message 2,over here\n")
        event = pipe.recv()
        assert event.kind is EventKind.MESSAGE
        assert event.text == "message 2,over here"
        assert event.command is None
        # The outstanding `see` is untouched, and its response still pairs.
        assert pipe.pending == 1
        peer.sendall(b"{player,,,}\n")
        resp = pipe.recv()
        assert resp.kind is EventKind.RESPONSE
        assert resp.command == "see"
    finally:
        peer.close()


def test_death_push_is_routed_not_matched() -> None:
    pipe, peer = _pipe()
    try:
        peer.sendall(b"death\n")
        event = pipe.recv()
        assert event.kind is EventKind.DEATH
        assert event.command is None
    finally:
        peer.close()


def test_unsolicited_response_line_does_not_underflow() -> None:
    pipe, peer = _pipe()
    try:
        # A response-shaped line with nothing outstanding must not pop a phantom.
        peer.sendall(b"ok\n")
        event = pipe.recv()
        assert event.kind is EventKind.UNSOLICITED
        assert event.text == "ok"
        assert pipe.pending == 0
    finally:
        peer.close()

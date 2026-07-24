"""Command sender / receiver with subject pipelining (C02 / RQ11, RQ12).

The autonomous AI (C03+) drives gameplay by *sending* protocol verbs and
*consuming* the server's replies. This module is the thin, testable layer in
between. It owns two subject-mandated invariants:

* **Exact command syntax (RQ11).** Every verb here is emitted precisely as the
  requirements table spells it — lowercase, space-separated arguments,
  ``\\n``-terminated (the newline is added by
  :meth:`~zappy_client.protocol.ServerConnection.send_line`). Building commands
  through the :class:`Command` constants and the :func:`pick` / :func:`drop` /
  :func:`broadcast` builders keeps typos out of the wire format so
  unknown/malformed lines — which the server answers with ``ko`` — never
  originate from us.

* **Pipeline depth ≤ 10 (RQ12).** The subject lets a client send up to 10
  requests without waiting for their responses; a server drops the 11th until a
  slot frees. :class:`CommandPipeline` mirrors that cap on the *client* side so
  we never speak commands the server will silently discard. :meth:`try_send`
  refuses to overflow the window and reports back; the caller decides whether to
  retry after draining a response.

Responses arrive in the same order the commands were sent (the server executes
one player's queue serially — SDS §3), so the pipeline matches each inbound
*response* line to the oldest outstanding command FIFO. Server-*initiated*
lines — ``death``, ``message <K>,<text>``, ``moving <K>`` — are not answers to
any command; they are routed out separately as :class:`ServerEvent` so a lone
push can never desynchronise the response/command pairing.
"""

from __future__ import annotations

from collections import deque
from dataclasses import dataclass
from enum import Enum, StrEnum

from .protocol import ServerConnection

# Subject pipeline window: at most this many requests may be outstanding
# (sent but unanswered) at once. The 11th is ignored by the server (RQ12).
MAX_PENDING = 10


class Command(StrEnum):
    """Zero-argument subject verbs, spelled exactly as the requirements table.

    :class:`~enum.StrEnum` members *are* their lowercase strings, so a member can
    be passed straight to :meth:`ServerConnection.send_line`, compared against
    raw wire text (``Command.SEE == "see"``), or ``str()``-formatted to ``"see"``.
    """

    ADVANCE = "advance"
    RIGHT = "right"
    LEFT = "left"
    SEE = "see"
    INVENTORY = "inventory"
    KICK = "kick"
    ENCHANTMENT = "enchantment"
    FORK = "fork"
    CONNECT_NBR = "connect_nbr"


def pick(obj: str) -> str:
    """Build a ``pick <object>`` command (RQ11).

    ``obj`` is a resource class name (``food`` or a stone type). It is inserted
    verbatim; callers are responsible for passing a valid class name.
    """
    return f"pick {obj}"


def drop(obj: str) -> str:
    """Build a ``drop <object>`` command (RQ11)."""
    return f"drop {obj}"


def broadcast(text: str) -> str:
    """Build a ``broadcast <text>`` command (RQ11).

    Newlines terminate protocol lines, so an embedded ``\\n`` would split one
    broadcast into two malformed commands. We reject it rather than emit a line
    the server would answer with ``ko``.
    """
    if "\n" in text:
        raise ValueError("broadcast text must not contain a newline")
    return f"broadcast {text}"


class EventKind(Enum):
    """Classification of an inbound server line."""

    RESPONSE = "response"  # reply to the oldest outstanding command
    DEATH = "death"  # server push: this player starved (RQ07)
    MESSAGE = "message"  # server push: broadcast heard (RQ15)
    MOVING = "moving"  # server push: kicked by another player (RQ14)
    UNSOLICITED = "unsolicited"  # a non-push line with no command waiting


@dataclass(frozen=True)
class ServerEvent:
    """One classified inbound line.

    ``command`` is the verb this line answers (only for
    :attr:`EventKind.RESPONSE`); ``text`` is the raw line with the newline
    already stripped.
    """

    kind: EventKind
    text: str
    command: str | None = None


def classify(line: str) -> EventKind:
    """Classify a server line as a push or a command response.

    Server-initiated pushes are the three lines the server can send without the
    client asking (SDS §3 / §8): ``death``, ``message <K>,<text>`` and
    ``moving <K>``. Everything else is treated as a command response.
    """
    if line == "death":
        return EventKind.DEATH
    if line.startswith("message "):
        return EventKind.MESSAGE
    if line.startswith("moving "):
        return EventKind.MOVING
    return EventKind.RESPONSE


class CommandPipeline:
    """Send commands and receive classified events over a :class:`ServerConnection`.

    Enforces the ≤10 outstanding-request window (RQ12) and keeps command/response
    pairing intact by routing server pushes aside (RQ11 responses stay in order).
    """

    def __init__(self, conn: ServerConnection, max_pending: int = MAX_PENDING) -> None:
        self._conn = conn
        self._max_pending = max_pending
        self._pending: deque[str] = deque()

    @property
    def pending(self) -> int:
        """Number of sent-but-unanswered commands (0..=``max_pending``)."""
        return len(self._pending)

    @property
    def has_capacity(self) -> bool:
        """True while another command may be sent without overflowing the window."""
        return len(self._pending) < self._max_pending

    def try_send(self, command: str) -> bool:
        """Send ``command`` if the pipeline window has a free slot.

        Returns ``True`` if the command was written and is now outstanding, or
        ``False`` if the window is full (10 in flight) — in which case nothing is
        sent, matching the server dropping an 11th request (RQ12). Drain a
        response with :meth:`recv` and retry.
        """
        if not self.has_capacity:
            return False
        self._conn.send_line(str(command))
        self._pending.append(str(command))
        return True

    def send(self, command: str) -> None:
        """Send ``command``, raising :class:`PipelineFull` if the window is full.

        Use when the caller has already checked :attr:`has_capacity` and treats
        an overflow as a programming error rather than back-pressure.
        """
        if not self.try_send(command):
            raise PipelineFull(f"pipeline full: {self._max_pending} requests already outstanding")

    def recv(self) -> ServerEvent:
        """Read the next line and classify it.

        A response line is paired FIFO with the oldest outstanding command and
        frees a pipeline slot. A server push (``death`` / ``message`` /
        ``moving``) is returned without touching the outstanding window. A
        non-push line that arrives with no command waiting is reported as
        :attr:`EventKind.UNSOLICITED` (it never pops a phantom command).
        """
        line = self._conn.recv_line()
        kind = classify(line)
        if kind is EventKind.RESPONSE:
            if not self._pending:
                return ServerEvent(EventKind.UNSOLICITED, line)
            command = self._pending.popleft()
            return ServerEvent(EventKind.RESPONSE, line, command=command)
        return ServerEvent(kind, line)


class PipelineFull(Exception):
    """Raised by :meth:`CommandPipeline.send` when the ≤10 window is full."""

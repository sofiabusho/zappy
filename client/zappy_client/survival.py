"""Autonomous survival AI: move, see, pick food, avoid death (C03).

This is the first ticket that actually *plays*. Everything below runs off the
server socket alone — the agent perceives only through ``see`` / ``inventory``
replies and acts only through subject commands. It never reads a file, opens a
side channel, or coordinates with another client out of band, satisfying the
"no data exchange outside the game" rule (RQ20).

Survival strategy (RQ07 — one food = 126 life TU, starvation kills):

* **Perceive.** Each cycle sends ``see`` and parses the visible tiles
  (:mod:`zappy_client.parsing`).
* **Eat what you stand on.** If food is on the current square, ``pick food``
  immediately — extra food is always more life, so there is never a reason to
  step off it hungry.
* **Steer to the nearest food.** Otherwise pick the closest visible food tile
  and walk a deterministic advance/turn plan onto it (:func:`plan_toward`), then
  eat.
* **Explore when blind.** With no food in view, wander (mostly forward, with the
  occasional turn) so the vision cone sweeps new ground instead of retracing a
  straight line.

The agent keeps **at most one command outstanding** — it sends, then drains the
reply before sending again — so it can never exceed the subject's ≤10 pipeline
window (RQ12) and its command/response pairing stays trivially in order. Server
pushes that are not replies (``message`` heard, ``moving`` when kicked) are
skipped; a ``death`` push ends the run cleanly (RQ07 / AQ07 observed from the
client side).
"""

from __future__ import annotations

import random
import sys
from collections.abc import Callable

from .parsing import FOOD, TileOffset, nearest_food, parse_inventory, parse_see, tile_offset
from .pipeline import Command, CommandPipeline, EventKind, ServerEvent, pick
from .protocol import ConnectionClosed

# How often (in perceive-act cycles) to read the inventory purely to observe
# remaining life for the operator/auditor. Gameplay does not depend on it.
DEFAULT_INVENTORY_PERIOD = 25

# Wander bias: probability of turning (each side) instead of advancing when no
# food is visible. Kept modest so exploration mostly moves forward.
_TURN_PROB_EACH_SIDE = 0.15


def plan_toward(offset: TileOffset) -> list[str]:
    """Return the advance/turn commands that land the player on ``offset``.

    Right-positive lateral: to reach a tile ``f`` forward and ``l`` to the side
    we sidestep first (turn toward the side, advance ``|l|``, turn back to the
    original facing) then advance ``f`` forward. On a torus these relative moves
    compose exactly, so the sequence always ends on the target square regardless
    of world size or wrapping. The looking player's own square yields ``[]``.
    """
    plan: list[str] = []
    if offset.lateral > 0:
        plan.append(Command.RIGHT)
        plan.extend([Command.ADVANCE] * offset.lateral)
        plan.append(Command.LEFT)
    elif offset.lateral < 0:
        plan.append(Command.LEFT)
        plan.extend([Command.ADVANCE] * (-offset.lateral))
        plan.append(Command.RIGHT)
    plan.extend([Command.ADVANCE] * offset.forward)
    return plan


def _default_log(message: str) -> None:
    print(f"client: {message}", file=sys.stderr)


class SurvivalAgent:
    """Drive an autonomous survival loop over a :class:`CommandPipeline`.

    Parameters
    ----------
    pipeline:
        Connected command pipeline (post-handshake).
    rng:
        Injectable :class:`random.Random` for deterministic wandering in tests.
    log:
        One-argument callback for human-readable status (defaults to stderr).
    inventory_period:
        Cycles between inventory life polls (``0`` disables polling).
    """

    def __init__(
        self,
        pipeline: CommandPipeline,
        *,
        rng: random.Random | None = None,
        log: Callable[[str], None] | None = None,
        inventory_period: int = DEFAULT_INVENTORY_PERIOD,
    ) -> None:
        self._pipeline = pipeline
        self._rng = rng if rng is not None else random.Random()
        self._log = log if log is not None else _default_log
        self._inventory_period = inventory_period
        self._alive = True
        self._cycle = 0

    @property
    def alive(self) -> bool:
        """False once the player has starved (a ``death`` push was seen)."""
        return self._alive

    # -- one perceive-act cycle -------------------------------------------------

    def step(self) -> bool:
        """Run one perceive-act cycle. Returns ``False`` when the player is gone.

        Order: eat food underfoot, else head for the nearest visible food, else
        explore. Any command whose reply is a ``death`` push terminates the
        cycle with ``False``.
        """
        see = self._send_await(Command.SEE)
        if see is None:
            return False
        try:
            tiles = parse_see(see)
        except ValueError:
            # A malformed view is non-fatal: skip this cycle and re-perceive.
            self._log(f"ignoring malformed see reply {see!r}")
            return self._alive

        if tiles and FOOD in tiles[0]:
            return self._send_await(pick(FOOD)) is not None

        index = nearest_food(tiles)
        if index is not None and index != 0:
            return self._seek_food(index)

        return self._wander() is not None

    def _seek_food(self, index: int) -> bool:
        """Walk onto the visible food tile at ``index`` and eat it."""
        for command in plan_toward(tile_offset(index)):
            if self._send_await(command) is None:
                return False
        return self._send_await(pick(FOOD)) is not None

    def _wander(self) -> str | None:
        """Take one exploratory step (mostly forward, sometimes a turn)."""
        roll = self._rng.random()
        if roll < _TURN_PROB_EACH_SIDE:
            command: str = Command.LEFT
        elif roll < 2 * _TURN_PROB_EACH_SIDE:
            command = Command.RIGHT
        else:
            command = Command.ADVANCE
        return self._send_await(command)

    # -- inventory life gauge (observation only) --------------------------------

    def poll_life(self) -> int | None:
        """Read the ``inventory`` and return remaining life in time units.

        ``food`` in the inventory reply is life TU (RQ07). Returns ``None`` if
        the player died mid-read or the reply was malformed.
        """
        reply = self._send_await(Command.INVENTORY)
        if reply is None:
            return None
        try:
            return parse_inventory(reply).get(FOOD)
        except ValueError:
            self._log(f"ignoring malformed inventory reply {reply!r}")
            return None

    # -- transport: one outstanding command, drain pushes -----------------------

    def _send_await(self, command: str) -> str | None:
        """Send ``command`` and return its response text (``None`` if dead).

        Only one command is ever in flight, so the ≤10 window (RQ12) can never be
        exceeded and the reply pairs unambiguously with this command.
        """
        self._pipeline.send(command)
        return self._await_response()

    def _await_response(self) -> str | None:
        """Return the next command *response*, routing pushes aside.

        ``death`` flips :attr:`alive` and returns ``None``; ``message`` /
        ``moving`` / stray lines are logged and skipped without consuming the
        outstanding command.
        """
        while True:
            event = self._pipeline.recv()
            if event.kind is EventKind.DEATH:
                self._alive = False
                self._log("player starved (death); ending run.")
                return None
            if event.kind is EventKind.RESPONSE:
                return event.text
            self._on_push(event)

    def _on_push(self, event: ServerEvent) -> None:
        """Handle a non-response line (broadcast heard, kicked, or a stray)."""
        if event.kind is EventKind.MESSAGE:
            # In-game sound only; C04 acts on meetup broadcasts. Survival ignores.
            self._log(f"heard broadcast: {event.text}")
        elif event.kind is EventKind.MOVING:
            self._log(f"kicked: {event.text}")
        else:
            self._log(f"unsolicited line: {event.text}")

    # -- driver -----------------------------------------------------------------

    def run(self, max_steps: int | None = None) -> int:
        """Loop until death, disconnect, or ``max_steps`` cycles elapse.

        Returns a process exit code: ``0`` for any clean end (death and server
        close are normal game outcomes), ``1`` on an unexpected transport error.
        """
        steps = 0
        while self._alive and (max_steps is None or steps < max_steps):
            try:
                self._maybe_poll_life()
                if not self.step():
                    break
            except ConnectionClosed:
                self._log("server closed the connection; exiting.")
                return 0
            except OSError as exc:
                self._log(f"transport error: {exc}; exiting.")
                return 1
            steps += 1
            self._cycle += 1
        return 0

    def _maybe_poll_life(self) -> None:
        if self._inventory_period and self._cycle > 0 and self._cycle % self._inventory_period == 0:
            life = self.poll_life()
            if life is not None:
                self._log(f"life remaining: {life} time units")

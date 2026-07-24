"""Gathering + broadcast meetup + enchantment + fork AI (C04, C05).

C03 kept the player *alive*; C04 makes it *evolve*; C05 lets it *grow the
family*. It extends the survival loop with the behaviours the subject's evolution
ritual demands (RQ09), all of them expressed purely through server commands so no
data ever crosses between clients out of band (RQ20):

* **Gather stones.** Beyond eating, the agent now picks up the stone classes its
  current level's ritual needs (:mod:`zappy_client.ritual`), walking to the
  nearest visible one when none is underfoot.
* **Broadcast a meetup.** Rituals above level 1 need several same-level players
  on one square. A player that already holds the required stones becomes a
  *beacon*: it stays put and periodically ``broadcast``s a rendezvous call
  (RQ15). Players that hear the call move toward the sound's direction ``K`` so
  the group converges — coordination achieved through the game's own sound
  channel, never a side channel.
* **Fork for family slots (C05).** A group ritual (level ≥2) needs 2, 4, or 6
  same-level players on one tile, and the win needs six teammates at level 8 — so
  a beacon short on partners must be able to *summon* more family members. When
  the beacon polls ``connect_nbr`` and finds no free slot for a teammate to join
  through, it sends ``fork`` (48/t) to call a ship; after 600/t the family gains
  a slot and a new member can connect (RQ13 / AQ26). Forking is throttled and
  yields to survival so a lone player never starves calling ships it cannot use.
* **Attempt the enchantment.** When the stones are in hand and enough same-level
  players share the tile, it sends ``enchantment`` and rides out the ritual,
  levelling up on the server's ``current level : K`` push (AQ25).

Survival still comes first — a starved player evolves nothing — so food underfoot
is always eaten and a low life gauge diverts the agent back to foraging.

The transport contract from C03 is unchanged: **one command outstanding at a
time** (never exceeding the ≤10 window, RQ12), with server *pushes* — ``death``,
``message <K>,<text>`` heard, and ``current level : K`` — routed aside from
command responses so pairing never desynchronises.
"""

from __future__ import annotations

import random
from collections.abc import Callable

from .parsing import FOOD, nearest_food, nearest_with, parse_see, tile_offset
from .pipeline import (
    EVOLUTION_IN_PROGRESS,
    Command,
    CommandPipeline,
    EventKind,
    ServerEvent,
    broadcast,
    parse_current_level,
    pick,
)
from .ritual import missing_stones, requirements_for
from .survival import SurvivalAgent, plan_toward

# Rendezvous tag prefix for meetup broadcasts. Any player hearing a broadcast
# that starts with this treats it as a call to gather; the trailing ``L<level>``
# lets a listener ignore calls for a level it cannot ritual with.
MEETUP_PREFIX = "zappy"

# Life (time units) at or below which the agent drops gathering and forages, so
# it never starves chasing stones. One food = 126 TU; this leaves a few moves of
# margin (RQ07).
DEFAULT_HUNGER_THRESHOLD = 300

# Cycles between meetup broadcasts while acting as a beacon (throttle so the
# sound channel is not spammed every 7/t).
DEFAULT_BROADCAST_PERIOD = 5

# Minimum cycles between ``fork`` attempts while a beacon is short on ritual
# partners. A ship takes 600/t to arrive, so we throttle well above the broadcast
# cadence: this prevents queuing more ships than the family can plausibly fill
# while still opening a slot when reinforcements are genuinely needed (RQ13).
DEFAULT_FORK_PERIOD = 30


def meetup_text(level: int) -> str:
    """Build the rendezvous broadcast payload for a given ``level``."""
    return f"{MEETUP_PREFIX} L{level}"


def is_meetup(text: str) -> bool:
    """True if a heard broadcast ``text`` is a rendezvous call."""
    return text.startswith(MEETUP_PREFIX)


def approach_plan(k: int) -> list[str]:
    """Move one square toward a sound arriving from sector ``K`` (0..8).

    ``K`` is the direction the sound *comes from*, relative to the listener's
    facing (front ``1``, left ``3``, back ``5``, right ``7``, counter-clockwise;
    ``0`` = same tile). We turn roughly toward that sector and advance, so
    repeated calls over successive broadcasts walk the listener onto the source's
    square. It is deliberately coarse — exact pathing is unnecessary because the
    beacon keeps broadcasting until the group forms.
    """
    if k == 0:
        return []  # already together
    if k in (8, 1, 2):
        return [Command.ADVANCE]
    if k in (3, 4):
        return [Command.LEFT, Command.ADVANCE]
    if k in (6, 7):
        return [Command.RIGHT, Command.ADVANCE]
    # k == 5: source is directly behind — turn around, then advance.
    return [Command.RIGHT, Command.RIGHT, Command.ADVANCE]


class GatheringAgent(SurvivalAgent):
    """Autonomous gatherer that forages, rallies teammates, and evolves.

    Builds on :class:`~zappy_client.survival.SurvivalAgent`, reusing its
    one-outstanding-command transport and wandering, and adds stone gathering,
    meetup broadcasting, and enchantment attempts.

    Parameters
    ----------
    pipeline, rng, log, inventory_period:
        As :class:`SurvivalAgent`.
    hunger_threshold:
        Life (TU) below which foraging preempts gathering.
    broadcast_period:
        Cycles between beacon broadcasts (``0`` disables broadcasting).
    fork_period:
        Minimum cycles between ``fork`` ship calls when a beacon needs more
        family members (``0`` disables forking).
    """

    def __init__(
        self,
        pipeline: CommandPipeline,
        *,
        rng: random.Random | None = None,
        log: Callable[[str], None] | None = None,
        inventory_period: int = 25,
        hunger_threshold: int = DEFAULT_HUNGER_THRESHOLD,
        broadcast_period: int = DEFAULT_BROADCAST_PERIOD,
        fork_period: int = DEFAULT_FORK_PERIOD,
    ) -> None:
        super().__init__(
            pipeline,
            rng=rng,
            log=log,
            inventory_period=inventory_period,
        )
        self._hunger_threshold = hunger_threshold
        self._broadcast_period = broadcast_period
        self._fork_period = fork_period
        # Stones we believe we hold, tracked locally from successful picks and
        # re-synced against the server on level-up. Players start with 0 (RQ06).
        self._stones: dict[str, int] = {}
        # Current level, advanced by the server's `current level : K` push (never
        # granted locally). Players start at level 1 (RQ06).
        self.level = 1
        # Best-known remaining life; updated by inventory polls. Optimistic start
        # (10 food × 126 TU) until the first poll.
        self._life = 1260
        # True while we are (or believe we are) inside an enchantment: stay put
        # so we remain on the ritual tile until it resolves.
        self._in_ritual = False
        # Last meetup sound direction we heard but have not yet acted on.
        self._heard_meetup_k: int | None = None
        self._last_broadcast_cycle: int | None = None
        # Cycle of our last `fork` attempt, throttling ship calls (C05 / RQ13).
        self._last_fork_cycle: int | None = None

    # -- perceive-act cycle -----------------------------------------------------

    def step(self) -> bool:
        """Run one gather/rally/ritual cycle; ``False`` when the player is gone."""
        if self._in_ritual:
            # Hold position and keep perceiving until the ritual push arrives.
            return self._perceive() is not None

        see = self._perceive()
        if see is None:
            return False
        if see == EVOLUTION_IN_PROGRESS:
            # We were swept into a ritual started by someone on our tile: stay.
            self._in_ritual = True
            self._log("joined an enchantment in progress; holding position.")
            return True
        try:
            tiles = parse_see(see)
        except ValueError:
            self._log(f"ignoring malformed see reply {see!r}")
            return self._alive
        if not tiles:
            return self._wander() is not None

        here = tiles[0]

        # 1. Survival always wins: eat food underfoot; forage when hungry.
        if FOOD in here:
            return self._send_await(pick(FOOD)) is not None
        if self._life <= self._hunger_threshold:
            return self._forage(tiles)

        # 2. Gather a needed stone underfoot.
        wanted = set(self._missing().keys())
        stone_here = next((s for s in here if s in wanted), None)
        if stone_here is not None:
            return self._pick_stone(stone_here)

        # 3. Stones complete → try to ritual, or rally partners for a group one.
        if not wanted:
            action = self._ritual_or_rally(tiles)
            if action is not None:
                return action

        # 4. Head for the nearest needed stone, else a heard meetup, else forage.
        if wanted:
            index = nearest_with(tiles, wanted)
            if index is not None and index != 0:
                return self._walk_to(index)
        if self._heard_meetup_k is not None:
            return self._go_to_meetup()
        return self._forage(tiles)

    # -- gathering / ritual / rally ---------------------------------------------

    def _missing(self) -> dict[str, int]:
        """Stones still needed for this level's ritual given local holdings."""
        return missing_stones(self._stones, self.level)

    def _pick_stone(self, stone: str) -> bool:
        """Pick a stone underfoot, crediting it locally only if the server oks it."""
        reply = self._send_await(pick(stone))
        if reply is None:
            return False
        if reply == "ok":
            self._stones[stone] = self._stones.get(stone, 0) + 1
            self._log(f"picked {stone} ({self._stones[stone]} held)")
        return True

    def _ritual_or_rally(self, tiles: list[list[str]]) -> bool | None:
        """Start the enchantment if the tile is ready, else beacon for partners.

        Returns the cycle result, or ``None`` if no ritual/rally action applies
        (so the caller can fall through to foraging).
        """
        req = requirements_for(self.level)
        if req is None:
            return None  # level 8: nothing left to ritual (RQ02 win handled server-side)
        others_here = tiles[0].count("player")
        if others_here + 1 >= req.players:
            return self._attempt_enchantment()
        # Not enough same-level players yet. A group ritual needs bodies the family
        # may not have, so first make sure a slot exists for a teammate to join —
        # forking a ship if none is free (C05 / RQ13). Both branches stay on this
        # tile so the beacon does not wander off before partners arrive.
        if req.players > 1 and self._due_to_fork():
            return self._maybe_fork()
        # Rally same-level players toward us (RQ15 meetup coordination).
        if self._broadcast_period and self._due_to_broadcast():
            return self._broadcast_meetup()
        # Stay put between broadcasts so arriving partners find us on this tile.
        return self._send_await(Command.SEE) is not None

    def _attempt_enchantment(self) -> bool:
        """Send ``enchantment`` and enter the ritual on the server's ack."""
        reply = self._send_await(Command.ENCHANTMENT)
        if reply is None:
            return False
        if reply == EVOLUTION_IN_PROGRESS:
            self._in_ritual = True
            self._log(f"enchantment started at level {self.level}.")
        else:
            # `ko`: not eligible (lost a partner/stone to a race). Retry later.
            self._log(f"enchantment refused ({reply!r}); will retry.")
        return True

    def _due_to_broadcast(self) -> bool:
        if self._last_broadcast_cycle is None:
            return True
        return self._cycle - self._last_broadcast_cycle >= self._broadcast_period

    def _broadcast_meetup(self) -> bool:
        """Broadcast a rendezvous call for same-level partners (RQ15)."""
        self._last_broadcast_cycle = self._cycle
        reply = self._send_await(broadcast(meetup_text(self.level)))
        if reply is None:
            return False
        self._log(f"broadcast meetup for level {self.level}.")
        return True

    def _go_to_meetup(self) -> bool:
        """Take one step toward the last heard meetup sound, then clear it."""
        k = self._heard_meetup_k
        self._heard_meetup_k = None
        if k is None:
            return True
        for command in approach_plan(k):
            if self._send_await(command) is None:
                return False
        return True

    # -- fork: grow the family when a group ritual is short on players -----------

    def _due_to_fork(self) -> bool:
        """True when a beacon should (re)check whether to summon a teammate.

        Gated so ship calls stay rare: forking is disabled when ``fork_period`` is
        zero, deferred while the player is hungry (survival outranks family growth,
        RQ07), and otherwise throttled to at most once per ``fork_period`` cycles
        so we never queue more ships than the family can fill (a ship needs 600/t
        to arrive).
        """
        if self._fork_period <= 0:
            return False
        if self._life <= self._hunger_threshold:
            return False
        if self._last_fork_cycle is None:
            return True
        return self._cycle - self._last_fork_cycle >= self._fork_period

    def _maybe_fork(self) -> bool:
        """Open a family slot for a missing ritual partner, forking if none is free.

        Polls ``connect_nbr`` for the team's free connection slots — a bare
        integer of the connections still authorized for this family (RQ13). A
        positive count means a teammate can already join to fill the gap, so we
        hold the tile and wait. Zero free slots means no reinforcement can arrive,
        so we ``fork`` (48/t) to call a ship: after 600/t the family gains a slot a
        new member can occupy (AQ26). Returns ``True`` while alive, ``False`` if a
        ``death`` push arrived mid-exchange (ending the run); a malformed
        ``connect_nbr`` reply is non-fatal and simply skips the fork this cycle.
        """
        self._last_fork_cycle = self._cycle
        reply = self._send_await(Command.CONNECT_NBR)
        if reply is None:
            return False  # died polling
        try:
            free = int(reply.strip())
        except ValueError:
            self._log(f"ignoring malformed connect_nbr reply {reply!r}")
            return self._alive
        if free > 0:
            self._log(f"{free} family slot(s) free; awaiting a teammate to rally.")
            return True
        fork_reply = self._send_await(Command.FORK)
        if fork_reply is None:
            return False
        self._log("forked: called a ship for a new family member.")
        return True

    # -- survival helpers reused for foraging -----------------------------------

    def _forage(self, tiles: list[list[str]]) -> bool:
        """Eat toward the nearest visible food, else wander (survival fallback)."""
        index = nearest_food(tiles)
        if index is not None and index != 0:
            return self._seek_food(index)
        return self._wander() is not None

    def _walk_to(self, index: int) -> bool:
        """Walk onto the visible tile at ``index`` (e.g. a needed stone)."""
        for command in plan_toward(tile_offset(index)):
            if self._send_await(command) is None:
                return False
        return True

    def _perceive(self) -> str | None:
        """Send ``see`` and return the raw reply (``None`` if the player died)."""
        return self._send_await(Command.SEE)

    # -- push handling: ritual level-ups and heard meetups ----------------------

    def _await_response(self) -> str | None:
        """Return the next command response, routing ``current level`` pushes aside.

        Extends the survival router so a ``current level : K`` push (a ritual
        completing) updates our level/inventory instead of being mistaken for a
        command reply.
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
        """Handle a non-response line (level-up, meetup heard, kicked, stray)."""
        if event.kind is EventKind.RITUAL_LEVEL:
            self._on_level_up(event.text)
        elif event.kind is EventKind.MESSAGE:
            self._on_message(event.text)
        elif event.kind is EventKind.MOVING:
            self._log(f"kicked: {event.text}")
        else:
            self._log(f"unsolicited line: {event.text}")

    def _on_level_up(self, line: str) -> None:
        """Apply a ``current level : K`` push: bump level, drop consumed stones."""
        new_level = parse_current_level(line)
        if new_level is None:
            self._log(f"ignoring malformed level push {line!r}")
            return
        consumed = requirements_for(self.level)
        if consumed is not None and new_level > self.level:
            for stone, count in consumed.stones.items():
                self._stones[stone] = max(0, self._stones.get(stone, 0) - count)
        self.level = new_level
        self._in_ritual = False
        self._log(f"evolved to level {new_level}.")

    def _on_message(self, line: str) -> None:
        """Record a heard meetup broadcast (``message <K>,<text>``) for next step."""
        # Format: `message <K>,<text>` (newline already stripped upstream).
        body = line[len("message ") :]
        sector, _, text = body.partition(",")
        if not is_meetup(text):
            self._log(f"heard broadcast: {line}")
            return
        try:
            k = int(sector.strip())
        except ValueError:
            self._log(f"heard meetup with bad direction: {line!r}")
            return
        self._heard_meetup_k = k
        self._log(f"heard meetup call from direction {k}.")

    # -- life gauge: cache the poll so hunger can gate gathering -----------------

    def poll_life(self) -> int | None:
        """Poll remaining life and cache it for hunger decisions."""
        life = super().poll_life()
        if life is not None:
            self._life = life
        return life

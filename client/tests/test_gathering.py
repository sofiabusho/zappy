"""Tests for the gathering / meetup / enchantment agent (C04).

Coverage: RQ09 (ritual), RQ15 (broadcast meetup), RQ20 (in-game only), AQ25
(perform the ritual / level up). The agent is driven through a scripted fake
connection (no sockets, no threads): we pre-queue the exact server replies the
deterministic agent consumes and assert on the commands it emits, exercising the
real :class:`CommandPipeline` (push routing, ≤10 window) underneath.
"""

from __future__ import annotations

import random

from zappy_client.gathering import (
    GatheringAgent,
    approach_plan,
    is_meetup,
    meetup_text,
)
from zappy_client.pipeline import Command, CommandPipeline
from zappy_client.protocol import ConnectionClosed


class FakeConn:
    """A :class:`ServerConnection` stand-in with a scripted reply queue."""

    def __init__(self, replies: list[str]) -> None:
        self.sent: list[str] = []
        self._replies = list(replies)

    def send_line(self, text: str) -> None:
        self.sent.append(text)

    def recv_line(self) -> str:
        if not self._replies:
            raise ConnectionClosed("scripted server exhausted")
        return self._replies.pop(0)


def _agent(replies: list[str], **kwargs: object) -> tuple[GatheringAgent, FakeConn]:
    conn = FakeConn(replies)
    logs: list[str] = []
    agent = GatheringAgent(
        CommandPipeline(conn),  # type: ignore[arg-type]
        log=logs.append,
        inventory_period=0,
        **kwargs,  # type: ignore[arg-type]
    )
    agent.logs = logs  # type: ignore[attr-defined]
    return agent, conn


# --- pure helpers --------------------------------------------------------------


def test_meetup_text_and_is_meetup_roundtrip() -> None:
    assert meetup_text(3) == "zappy L3"
    assert is_meetup(meetup_text(3)) is True
    assert is_meetup("hello there") is False


def test_approach_plan_directions() -> None:
    assert approach_plan(0) == []
    assert approach_plan(1) == [Command.ADVANCE]  # front
    assert approach_plan(8) == [Command.ADVANCE]
    assert approach_plan(3) == [Command.LEFT, Command.ADVANCE]  # left
    assert approach_plan(7) == [Command.RIGHT, Command.ADVANCE]  # right
    assert approach_plan(5) == [Command.RIGHT, Command.RIGHT, Command.ADVANCE]  # behind


# --- gathering -----------------------------------------------------------------


def test_picks_needed_stone_underfoot() -> None:
    # Level 1 needs jade; a jade sits on the current tile.
    agent, conn = _agent(["{jade, , , }", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "pick jade"]
    assert agent._stones == {"jade": 1}  # type: ignore[attr-defined]


def test_ignores_stone_not_needed_for_this_level() -> None:
    # Level 1 only needs jade; an amber underfoot is not gathered — the agent
    # falls through to foraging/wandering instead of picking it.
    agent, conn = _agent(["{amber, , , }", "ok"], rng=random.Random(0))
    assert agent.step() is True
    assert conn.sent[0] == "see"
    assert conn.sent[1] != "pick amber"


def test_walks_toward_visible_needed_stone() -> None:
    # Jade one tile ahead (index 2); no stone underfoot -> navigate onto it.
    agent, conn = _agent(["{, , jade, }", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "advance"]


# --- solo enchantment + level-up ----------------------------------------------


def test_solo_ritual_starts_when_stones_ready() -> None:
    agent, conn = _agent(["{, , , }", "evolution in progress"])
    agent._stones = {"jade": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "enchantment"]
    assert agent._in_ritual is True  # type: ignore[attr-defined]


def test_level_up_push_advances_level_and_consumes_stones() -> None:
    # In-ritual: perceive, receive the `current level : 2` push then the see reply.
    agent, conn = _agent(["current level : 2", "{, , , }"])
    agent._stones = {"jade": 1}  # type: ignore[attr-defined]
    agent._in_ritual = True  # type: ignore[attr-defined]
    assert agent.step() is True
    assert agent.level == 2
    assert agent._in_ritual is False  # type: ignore[attr-defined]
    assert agent._stones.get("jade", 0) == 0  # type: ignore[attr-defined]  # consumed
    assert conn.sent == ["see"]


def test_enchantment_refusal_is_retried_not_fatal() -> None:
    agent, conn = _agent(["{, , , }", "ko"])
    agent._stones = {"jade": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert agent._in_ritual is False  # type: ignore[attr-defined]
    assert any("refused" in line for line in agent.logs)  # type: ignore[attr-defined]


# --- multi-player meetup -------------------------------------------------------


def test_broadcasts_meetup_when_partners_needed() -> None:
    # Level 2 ritual needs 2 players; alone with full stones -> beacon broadcast.
    # fork_period=0 isolates the C04 rally path (fork strategy is exercised in the
    # C05 fork tests below).
    agent, conn = _agent(["{, , , }", "ok"], fork_period=0)
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "broadcast zappy L2"]


def test_starts_ritual_when_partner_shares_tile() -> None:
    # Level 2, full stones, one other player on the tile -> enough to enchant.
    agent, conn = _agent(["{player, , , }", "evolution in progress"])
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "enchantment"]
    assert agent._in_ritual is True  # type: ignore[attr-defined]


def test_joins_enchantment_started_by_another() -> None:
    # A bystander swept into a ritual sees `evolution in progress` as its `see`
    # reply; it must hold position, not treat it as a malformed view.
    agent, conn = _agent(["evolution in progress"])
    assert agent.step() is True
    assert agent._in_ritual is True  # type: ignore[attr-defined]
    assert conn.sent == ["see"]


def test_heard_meetup_drives_movement_toward_sound() -> None:
    # Still needs stones, none visible, but hears a meetup from the front (K=1):
    # steps toward the sound so the group can converge.
    agent, conn = _agent(["message 1,zappy L2", "{, , , }", "ok"])
    agent.level = 2
    assert agent.step() is True
    assert conn.sent == ["see", "advance"]
    assert agent._heard_meetup_k is None  # type: ignore[attr-defined]  # consumed


def test_non_meetup_broadcast_is_ignored_for_movement() -> None:
    agent, conn = _agent(["message 3,hello", "{, , , }", "ok"], rng=random.Random(0))
    agent.level = 2
    assert agent.step() is True
    assert agent._heard_meetup_k is None  # type: ignore[attr-defined]
    # No meetup stored -> it forages/wanders rather than homing on the sound.
    assert conn.sent[0] == "see"


# --- fork: grow the family when a group ritual is short on players (C05) --------


def test_forks_when_beacon_short_on_partners_and_no_free_slot() -> None:
    # Level 2 ritual needs 2 players; alone with full stones and no free family
    # slot (connect_nbr -> 0): summon a teammate by calling a ship (RQ13/AQ26).
    agent, conn = _agent(["{, , , }", "0", "ok"])
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "connect_nbr", "fork"]
    assert any("forked" in line for line in agent.logs)  # type: ignore[attr-defined]


def test_skips_fork_when_a_slot_is_already_free() -> None:
    # A free slot (connect_nbr -> 1) means a teammate can already join: hold the
    # tile and wait rather than queue a redundant ship.
    agent, conn = _agent(["{, , , }", "1"])
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "connect_nbr"]
    assert "fork" not in conn.sent


def test_fork_disabled_falls_back_to_broadcast() -> None:
    # fork_period=0 disables ship calls: the beacon rallies via broadcast instead.
    agent, conn = _agent(["{, , , }", "ok"], fork_period=0)
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "broadcast zappy L2"]
    assert "connect_nbr" not in conn.sent


def test_fork_is_throttled_between_attempts() -> None:
    # Having just forked, a follow-up cycle within fork_period must not fork again;
    # it rallies with a broadcast instead (avoids queuing more ships than fillable).
    agent, conn = _agent(["{, , , }", "ok"], fork_period=10)
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    agent._last_fork_cycle = 0  # type: ignore[attr-defined]  # forked at cycle 0
    # _cycle stays 0 (< fork_period since last fork) -> not due to fork.
    assert agent.step() is True
    assert conn.sent == ["see", "broadcast zappy L2"]


def test_solo_ritual_never_forks() -> None:
    # Level 1 ritual needs a single player: no partners to summon, so no ship call.
    agent, conn = _agent(["{, , , }", "evolution in progress"])
    agent._stones = {"jade": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "enchantment"]
    assert "connect_nbr" not in conn.sent


def test_malformed_connect_nbr_is_non_fatal() -> None:
    # A non-integer connect_nbr reply is ignored (skip the fork), not treated as
    # death: the agent survives the cycle.
    agent, conn = _agent(["{, , , }", "not-a-number"])
    agent.level = 2
    agent._stones = {"jade": 1, "peridot": 1, "amber": 1}  # type: ignore[attr-defined]
    assert agent.step() is True
    assert conn.sent == ["see", "connect_nbr"]
    assert agent.alive is True
    assert any("malformed connect_nbr" in line for line in agent.logs)  # type: ignore[attr-defined]


# --- survival still takes priority ---------------------------------------------


def test_eats_food_underfoot_before_gathering() -> None:
    # Food and a needed jade share the tile: eat first (survival wins).
    agent, conn = _agent(["{food jade, , , }", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "pick food"]


def test_low_life_forages_instead_of_gathering() -> None:
    # Hungry (life below threshold) with a needed jade ahead: seek food, not stone.
    # Food straight ahead (index 2), jade to the side (index 3).
    agent, conn = _agent(["{, , food, jade}", "ok", "ok"])
    agent._life = 100  # type: ignore[attr-defined]  # below hunger threshold
    assert agent.step() is True
    # Heads straight to the food and eats it; never picks the stone.
    assert conn.sent == ["see", "advance", "pick food"]
    assert "pick jade" not in conn.sent


# --- pipeline discipline -------------------------------------------------------


def test_run_keeps_pipeline_within_window() -> None:
    # Wander/forage cycles: one command outstanding at a time -> never exceeds 10.
    replies = ["{, , , }", "ok"] * 4
    agent, _ = _agent(replies, rng=random.Random(2))
    agent.run(max_steps=4)
    assert agent._pipeline.pending == 0  # type: ignore[attr-defined]


def test_death_push_ends_run_cleanly() -> None:
    agent, conn = _agent(["death"])
    assert agent.run() == 0
    assert agent.alive is False
    assert conn.sent == ["see"]

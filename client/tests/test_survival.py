"""Tests for the autonomous survival agent (C03 / RQ07, RQ20, AQ23).

The agent is driven through a scripted fake connection (no threads, no real
socket): we pre-queue the exact server replies the deterministic agent will
consume and assert on the commands it emits. This exercises the real
:class:`CommandPipeline` (push routing, ≤10 window) underneath.
"""

from __future__ import annotations

import random

import pytest

from zappy_client.parsing import TileOffset, tile_offset
from zappy_client.pipeline import CommandPipeline
from zappy_client.protocol import ConnectionClosed
from zappy_client.survival import SurvivalAgent, plan_toward


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


def _agent(replies: list[str], **kwargs: object) -> tuple[SurvivalAgent, FakeConn]:
    conn = FakeConn(replies)
    logs: list[str] = []
    agent = SurvivalAgent(
        CommandPipeline(conn),  # type: ignore[arg-type]
        log=logs.append,
        **kwargs,  # type: ignore[arg-type]
    )
    agent.logs = logs  # type: ignore[attr-defined]
    return agent, conn


# --- plan_toward ---------------------------------------------------------------


def test_plan_toward_current_tile_is_empty() -> None:
    assert plan_toward(TileOffset(0, 0)) == []


def test_plan_toward_straight_ahead_advances() -> None:
    assert plan_toward(TileOffset(3, 0)) == ["advance", "advance", "advance"]


def test_plan_toward_right_sidesteps_then_forward() -> None:
    # forward 1, lateral +1 -> turn right, advance, turn back, advance forward.
    assert plan_toward(tile_offset(3)) == ["right", "advance", "left", "advance"]


def test_plan_toward_left_sidesteps_then_forward() -> None:
    # forward 1, lateral -1 (index 1) -> turn left, advance, turn back, advance.
    assert plan_toward(tile_offset(1)) == ["left", "advance", "right", "advance"]


# --- step: eat, seek, wander ---------------------------------------------------


def test_step_eats_food_on_current_tile() -> None:
    agent, conn = _agent(["{food, , , }", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "pick food"]
    assert agent.alive is True


def test_step_navigates_to_food_straight_ahead_then_eats() -> None:
    # Food at index 2 = one tile forward.
    agent, conn = _agent(["{, , food, }", "ok", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "advance", "pick food"]


def test_step_navigates_to_side_food_then_eats() -> None:
    # Food at index 3 = forward 1, right 1.
    agent, conn = _agent(["{, , , food}", "ok", "ok", "ok", "ok", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "right", "advance", "left", "advance", "pick food"]


def test_step_wanders_when_no_food_visible() -> None:
    # Seeded rng makes the exploratory choice deterministic.
    agent, conn = _agent(["{, , , }", "ok"], rng=random.Random(0))
    assert agent.step() is True
    assert conn.sent[0] == "see"
    assert conn.sent[1] in {"advance", "left", "right"}


def test_step_ignores_malformed_see_without_dying() -> None:
    agent, conn = _agent(["not-a-view"])
    assert agent.step() is True  # skipped, still alive
    assert conn.sent == ["see"]


# --- death and pushes ----------------------------------------------------------


def test_death_push_ends_the_run() -> None:
    agent, conn = _agent(["death"])
    assert agent.step() is False
    assert agent.alive is False


def test_death_after_moving_stops_mid_plan() -> None:
    # Starves while walking toward food: advance ok, then death.
    agent, conn = _agent(["{, , food, }", "death"])
    assert agent.step() is False
    assert agent.alive is False
    assert conn.sent == ["see", "advance"]


def test_broadcast_push_is_skipped_not_treated_as_response() -> None:
    # A message heard mid-flight must not be mistaken for the `see` reply.
    agent, conn = _agent(["message 2,over here", "{food, , , }", "ok"])
    assert agent.step() is True
    assert conn.sent == ["see", "pick food"]
    assert any("over here" in line for line in agent.logs)  # type: ignore[attr-defined]


def test_moving_push_is_skipped() -> None:
    agent, conn = _agent(["moving 5", "{, , , }", "ok"], rng=random.Random(1))
    assert agent.step() is True
    assert any("kicked" in line for line in agent.logs)  # type: ignore[attr-defined]


# --- inventory life gauge ------------------------------------------------------


def test_poll_life_reads_food_as_time_units() -> None:
    agent, conn = _agent(
        ["{food 1260, jade 0, peridot 0, amber 0, amethyst 0, garnet 0, ammolite 0}"]
    )
    assert agent.poll_life() == 1260
    assert conn.sent == ["inventory"]


def test_poll_life_returns_none_on_death() -> None:
    agent, _ = _agent(["death"])
    assert agent.poll_life() is None
    assert agent.alive is False


# --- run driver ----------------------------------------------------------------


def test_run_stops_after_max_steps() -> None:
    # Three wander cycles: each is a `see` + one move. Inventory polling off.
    replies = ["{, , , }", "ok"] * 3
    agent, conn = _agent(replies, rng=random.Random(0), inventory_period=0)
    assert agent.run(max_steps=3) == 0
    assert conn.sent.count("see") == 3


def test_run_returns_zero_on_disconnect() -> None:
    # Reply queue runs dry -> ConnectionClosed -> clean exit (server closed).
    agent, _ = _agent(["{, , , }", "ok"], rng=random.Random(0), inventory_period=0)
    assert agent.run() == 0


def test_run_ends_on_death() -> None:
    agent, conn = _agent(["death"], inventory_period=0)
    assert agent.run() == 0
    assert agent.alive is False
    assert conn.sent == ["see"]


def test_run_keeps_pipeline_within_window() -> None:
    # One command outstanding at a time -> pipeline never exceeds the ≤10 cap.
    replies = ["{, , , }", "ok"] * 5
    agent, _ = _agent(replies, rng=random.Random(3), inventory_period=0)
    agent.run(max_steps=5)
    assert agent._pipeline.pending == 0  # type: ignore[attr-defined]


def test_run_polls_life_on_period() -> None:
    # With period 1, every cycle starts by reading inventory (poll happens for
    # cycle>0), so cycle 2 issues an `inventory` before its `see`.
    inv = "{food 1000, jade 0, peridot 0, amber 0, amethyst 0, garnet 0, ammolite 0}"
    replies = ["{, , , }", "ok", inv, "{, , , }", "ok"]
    agent, conn = _agent(replies, rng=random.Random(0), inventory_period=1)
    agent.run(max_steps=2)
    assert "inventory" in conn.sent


@pytest.mark.parametrize("index", [0, 1, 2, 3, 6, 8])
def test_plan_toward_is_reversible_length(index: int) -> None:
    # A plan's length equals the tile's step distance plus the two turns needed
    # to sidestep (0 turns when already aligned).
    off = tile_offset(index)
    plan = plan_toward(off)
    turns = 0 if off.lateral == 0 else 2
    assert len(plan) == off.steps + turns

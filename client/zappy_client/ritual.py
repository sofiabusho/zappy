"""Client-side ritual knowledge: stone types + the evolution table (C04).

The autonomous gathering AI needs to know, for its *current* level, exactly
which stones and how many co-located same-level players an enchantment requires
(RQ09). This module mirrors the subject table — the single source of truth is
``docs/raw/requirements.md`` and the server's ``server/src/ritual.rs``; the two
must stay identical, so the values here are asserted against the subject in the
tests. The client never trusts this table to *grant* a level-up (only the server
does that); it uses it purely to decide what to gather and when it is worth
sending ``enchantment`` (AQ25).

The six stone classes are listed in **subject order** (jade, peridot, amber,
amethyst, garnet, ammolite). ``food`` is deliberately *not* a stone — it is
life, handled by the survival layer.
"""

from __future__ import annotations

from dataclasses import dataclass

# Six stone classes, in the subject's column order (requirements.md "Resources"
# and the ritual table). Order is not semantically required by the client, but
# keeping it stable makes gathering priorities and logs deterministic.
STONE_NAMES: tuple[str, ...] = (
    "jade",
    "peridot",
    "amber",
    "amethyst",
    "garnet",
    "ammolite",
)

# Highest reachable level; a level-8 player has no further ritual (RQ02 win).
MAX_LEVEL = 8


@dataclass(frozen=True)
class RitualReq:
    """Requirements to evolve **from** ``from_level`` to ``from_level + 1``.

    ``players`` is the number of same-level players that must stand on the tile
    (the starter included); ``stones`` maps each needed stone class to its count
    (classes needing zero are omitted, so the mapping doubles as the shopping
    list the gatherer works down).
    """

    from_level: int
    players: int
    stones: dict[str, int]


def _req(from_level: int, players: int, counts: tuple[int, ...]) -> RitualReq:
    stones = {name: n for name, n in zip(STONE_NAMES, counts, strict=True) if n}
    return RitualReq(from_level=from_level, players=players, stones=stones)


# Subject ritual table (requirements.md "Evolution ritual"; mirrors
# server/src/ritual.rs RITUAL_TABLE). Counts are (jade, peridot, amber,
# amethyst, garnet, ammolite).
RITUAL_TABLE: dict[int, RitualReq] = {
    1: _req(1, 1, (1, 0, 0, 0, 0, 0)),
    2: _req(2, 2, (1, 1, 1, 0, 0, 0)),
    3: _req(3, 2, (2, 0, 1, 0, 2, 0)),
    4: _req(4, 4, (1, 1, 2, 0, 1, 0)),
    5: _req(5, 4, (1, 2, 1, 3, 0, 0)),
    6: _req(6, 6, (1, 2, 3, 0, 1, 0)),
    7: _req(7, 6, (2, 2, 2, 2, 2, 1)),
}


def requirements_for(level: int) -> RitualReq | None:
    """Requirements to evolve from ``level``; ``None`` at/above the cap (8)."""
    return RITUAL_TABLE.get(level)


def missing_stones(inventory: dict[str, int], level: int) -> dict[str, int]:
    """Stones still needed for ``level``'s ritual given a held ``inventory``.

    Returns ``{stone: shortfall}`` for every class the player is short on (a
    positive count). Empty means the stone half of the ritual is satisfied — the
    player can look for enough same-level partners and start the enchantment.
    ``level`` with no ritual (≥8) needs nothing.
    """
    req = requirements_for(level)
    if req is None:
        return {}
    short: dict[str, int] = {}
    for stone, need in req.stones.items():
        have = inventory.get(stone, 0)
        if have < need:
            short[stone] = need - have
    return short


def has_required_stones(inventory: dict[str, int], level: int) -> bool:
    """True when ``inventory`` already holds every stone ``level``'s ritual needs."""
    return not missing_stones(inventory, level)

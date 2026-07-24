"""Tests for the client-side ritual table + gathering helpers (C04 / RQ09).

The table here must stay byte-for-byte with the subject
(``docs/raw/requirements.md``) and the server (``server/src/ritual.rs``); these
assertions are the client's half of AQ31 (ritual rules match the subject).
"""

from __future__ import annotations

from zappy_client.ritual import (
    MAX_LEVEL,
    RITUAL_TABLE,
    STONE_NAMES,
    has_required_stones,
    missing_stones,
    requirements_for,
)


def test_six_stone_names_in_subject_order() -> None:
    assert STONE_NAMES == ("jade", "peridot", "amber", "amethyst", "garnet", "ammolite")


def test_table_covers_levels_one_through_seven() -> None:
    assert sorted(RITUAL_TABLE) == [1, 2, 3, 4, 5, 6, 7]
    assert requirements_for(MAX_LEVEL) is None  # level 8 has no further ritual


def test_level1_is_solo_with_one_jade() -> None:
    req = requirements_for(1)
    assert req is not None
    assert req.players == 1
    assert req.stones == {"jade": 1}


def test_full_table_matches_subject() -> None:
    # (players, {stone: count}) exactly per the subject evolution table.
    expected = {
        1: (1, {"jade": 1}),
        2: (2, {"jade": 1, "peridot": 1, "amber": 1}),
        3: (2, {"jade": 2, "amber": 1, "garnet": 2}),
        4: (4, {"jade": 1, "peridot": 1, "amber": 2, "garnet": 1}),
        5: (4, {"jade": 1, "peridot": 2, "amber": 1, "amethyst": 3}),
        6: (6, {"jade": 1, "peridot": 2, "amber": 3, "garnet": 1}),
        7: (6, {"jade": 2, "peridot": 2, "amber": 2, "amethyst": 2, "garnet": 2, "ammolite": 1}),
    }
    for level, (players, stones) in expected.items():
        req = requirements_for(level)
        assert req is not None
        assert req.players == players, level
        assert req.stones == stones, level


def test_missing_stones_reports_shortfall() -> None:
    # Level 2 needs jade+peridot+amber; holding one jade leaves the other two.
    assert missing_stones({"jade": 1}, 2) == {"peridot": 1, "amber": 1}


def test_missing_stones_empty_when_satisfied() -> None:
    inv = {"jade": 5, "peridot": 1, "amber": 1}
    assert missing_stones(inv, 2) == {}
    assert has_required_stones(inv, 2) is True


def test_missing_stones_empty_at_max_level() -> None:
    assert missing_stones({}, MAX_LEVEL) == {}
    assert has_required_stones({}, MAX_LEVEL) is True


def test_has_required_stones_false_when_short() -> None:
    assert has_required_stones({}, 1) is False
    assert has_required_stones({"jade": 1}, 1) is True

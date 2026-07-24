"""Tests for ``see`` / ``inventory`` parsing and vision geometry (C03).

These lock the wire formats emitted by ``server/src/vision.rs`` (see) and
``server/src/player.rs`` (inventory) and verify the index→offset arithmetic the
survival AI uses to locate food.
"""

from __future__ import annotations

import math

import pytest

from zappy_client.parsing import (
    TileOffset,
    nearest_food,
    parse_inventory,
    parse_see,
    tile_offset,
)

# --- parse_see -----------------------------------------------------------------


def test_parse_see_subject_example() -> None:
    # From requirements.md §Vision: "{food, player amber, garnet garnet, }".
    tiles = parse_see("{food, player amber, garnet garnet, }")
    assert tiles == [["food"], ["player", "amber"], ["garnet", "garnet"], []]


def test_parse_see_bare_level1_view_is_four_empty_tiles() -> None:
    assert parse_see("{, , , }") == [[], [], [], []]


def test_parse_see_keeps_duplicate_tokens() -> None:
    # Two garnets on a tile are both listed (objects of a class are indistinct).
    assert parse_see("{garnet garnet}") == [["garnet", "garnet"]]


def test_parse_see_rejects_non_brace_payload() -> None:
    with pytest.raises(ValueError):
        parse_see("food, ")


# --- parse_inventory -----------------------------------------------------------


def test_parse_inventory_reads_life_and_stones() -> None:
    inv = parse_inventory(
        "{food 1260, jade 0, peridot 0, amber 0, amethyst 0, garnet 0, ammolite 0}"
    )
    assert inv["food"] == 1260  # life TU at spawn (10 food × 126)
    assert inv["ammolite"] == 0


def test_parse_inventory_rejects_malformed_entry() -> None:
    with pytest.raises(ValueError):
        parse_inventory("{food}")


def test_parse_inventory_rejects_non_integer_count() -> None:
    with pytest.raises(ValueError):
        parse_inventory("{food lots}")


# --- tile_offset (vision triangle geometry) ------------------------------------


def test_tile_offset_matches_server_level1_layout() -> None:
    # vision.rs level-1 north: [self, left-forward, forward, right-forward].
    assert tile_offset(0) == TileOffset(0, 0)
    assert tile_offset(1) == TileOffset(1, -1)  # forward 1, left
    assert tile_offset(2) == TileOffset(1, 0)  # forward 1, centre
    assert tile_offset(3) == TileOffset(1, 1)  # forward 1, right


def test_tile_offset_depth_two_row() -> None:
    # Depth 2 occupies indices 4..8 with lateral -2..+2.
    assert tile_offset(4) == TileOffset(2, -2)
    assert tile_offset(6) == TileOffset(2, 0)
    assert tile_offset(8) == TileOffset(2, 2)


def test_tile_offset_depth_starts_at_square_index() -> None:
    for depth in range(1, 6):
        start = depth * depth
        assert tile_offset(start).forward == depth
        assert tile_offset(start).lateral == -depth


def test_tile_offset_steps_is_manhattan_distance() -> None:
    assert tile_offset(8).steps == 2 + 2
    assert tile_offset(0).steps == 0


def test_tile_offset_rejects_negative_index() -> None:
    with pytest.raises(ValueError):
        tile_offset(-1)


def test_row_widths_follow_two_d_plus_one() -> None:
    # Total tiles up to level L is (L+1)^2; each depth row is 2d+1 wide.
    for level in range(1, 4):
        total = (level + 1) ** 2
        depths = [tile_offset(i).forward for i in range(total)]
        for d in range(level + 1):
            width = 1 if d == 0 else 2 * d + 1
            assert depths.count(d) == width
        assert max(depths) == level
        assert math.isqrt(total - 1) == level


# --- nearest_food --------------------------------------------------------------


def test_nearest_food_none_when_absent() -> None:
    assert nearest_food([[], [], [], []]) is None


def test_nearest_food_prefers_closest_tile() -> None:
    # Food at index 2 (forward 1) is closer than index 8 (forward 2).
    tiles = [[] for _ in range(9)]
    tiles[8] = ["food"]
    tiles[2] = ["food"]
    assert nearest_food(tiles) == 2


def test_nearest_food_ties_break_to_lower_index() -> None:
    tiles = [[], ["food"], [], ["food"]]  # indices 1 and 3 both 1 step away
    assert nearest_food(tiles) == 1


def test_nearest_food_finds_current_tile() -> None:
    assert nearest_food([["food"], [], [], []]) == 0

"""Parse ``see`` / ``inventory`` payloads and reason about vision geometry (C03).

The survival AI (this ticket) turns the server's two *read* replies into game
state it can act on:

* :func:`parse_see` splits a ``{tile0, tile1, ...}`` line into a list of tiles,
  each a list of resource-class tokens. The wire format is fixed by
  ``server/src/vision.rs``: tiles are separated by ``", "``; the tokens inside a
  tile are space-separated (players first as ``player``, then ``food``, then
  stone names); an empty tile is the empty string.
* :func:`parse_inventory` splits ``{food N, jade N, ...}`` into a ``{name: int}``
  mapping. Per ``server/src/player.rs`` the ``food`` count is remaining life in
  time units (10 food × 126 TU = 1260 at spawn), so it doubles as the survival
  gauge the AI watches (RQ07).

:func:`tile_offset` maps a ``see`` tile index to its position relative to the
looking player — how many squares *forward* and how many to the *side* (right
positive). The server lays tiles out as a forward triangle (``vision.rs``): tile
``0`` is the current square, then each depth ``d`` contributes a row of ``2d+1``
tiles left-to-right. Depth ``d`` therefore starts at index ``d²`` and the lateral
offset within that row runs ``-d .. +d``. This is purely index arithmetic — it
does not depend on the player's level or facing — so the AI can locate visible
food and steer toward it (:func:`zappy_client.survival`).
"""

from __future__ import annotations

import math
from dataclasses import dataclass

FOOD = "food"


def _strip_braces(line: str) -> str:
    """Return the body inside ``{...}``; raise on a malformed payload."""
    text = line.strip()
    if not (text.startswith("{") and text.endswith("}")):
        raise ValueError(f"expected a {{...}} payload, got {line!r}")
    return text[1:-1]


def parse_see(line: str) -> list[list[str]]:
    """Parse a ``see`` reply into a list of tiles (each a list of tokens).

    Tiles are separated by ``", "`` and preserve order — index 0 is the looking
    player's own square. Tokens within a tile are whitespace-separated; an empty
    tile yields ``[]``. Example::

        "{food, player amber, garnet garnet, }"
        -> [["food"], ["player", "amber"], ["garnet", "garnet"], []]

    Duplicate tokens are kept (two ``garnet`` = two garnets), matching the
    subject's "if there is more than 1 object in a square, they are all
    indicated".
    """
    body = _strip_braces(line)
    # A single empty tile ("{}") still means one tile with no contents.
    return [segment.split() for segment in body.split(",")]


def parse_inventory(line: str) -> dict[str, int]:
    """Parse an ``inventory`` reply into ``{resource: count}``.

    ``food`` is remaining life in time units (RQ07). Entries are ``name N``
    pairs separated by ``", "``. Unknown/short entries raise :class:`ValueError`
    rather than silently corrupt the survival gauge.
    """
    body = _strip_braces(line)
    inventory: dict[str, int] = {}
    for entry in body.split(","):
        parts = entry.split()
        if len(parts) != 2:
            raise ValueError(f"malformed inventory entry {entry!r} in {line!r}")
        name, count = parts
        try:
            inventory[name] = int(count)
        except ValueError as exc:
            raise ValueError(f"non-integer count in inventory entry {entry!r}") from exc
    return inventory


@dataclass(frozen=True)
class TileOffset:
    """Position of a ``see`` tile relative to the looking player.

    ``forward`` squares are ahead of the player (0 = current square); ``lateral``
    is the sideways offset with **right positive**, **left negative**. The
    Chebyshev/`step` distance to reach the tile is ``forward + abs(lateral)``
    when navigating by advance/turn primitives.
    """

    forward: int
    lateral: int

    @property
    def steps(self) -> int:
        """Number of advance/turn moves to stand on this tile."""
        return self.forward + abs(self.lateral)


def tile_offset(index: int) -> TileOffset:
    """Map a ``see`` tile index to its :class:`TileOffset` (forward, lateral).

    Depth ``d`` occupies indices ``d² .. d²+2d`` (a row of ``2d+1`` tiles); the
    lateral offset is the position within that row shifted so the centre is 0.
    Index 0 is the current square ``(0, 0)``.
    """
    if index < 0:
        raise ValueError(f"tile index must be non-negative, got {index}")
    depth = math.isqrt(index)
    lateral = index - depth * depth - depth
    return TileOffset(forward=depth, lateral=lateral)


def nearest_food(tiles: list[list[str]]) -> int | None:
    """Return the index of the closest tile containing food, or ``None``.

    Distance is :attr:`TileOffset.steps`; ties break toward the lower index
    (which the triangle lays out left-before-right, nearer-before-farther), so
    the choice is deterministic and the AI never oscillates between two equally
    near foods.
    """
    return nearest_with(tiles, {FOOD})


def nearest_with(tiles: list[list[str]], wanted: set[str]) -> int | None:
    """Return the index of the closest tile containing any token in ``wanted``.

    Generalises :func:`nearest_food` so the gathering AI (C04) can steer toward
    the nearest tile holding a stone it still needs. Distance is
    :attr:`TileOffset.steps`; ties break toward the lower index, keeping the
    choice deterministic so the agent never oscillates between two equally near
    tiles.
    """
    best: int | None = None
    best_steps = 0
    for index, tokens in enumerate(tiles):
        if wanted.isdisjoint(tokens):
            continue
        steps = tile_offset(index).steps
        if best is None or steps < best_steps:
            best, best_steps = index, steps
    return best

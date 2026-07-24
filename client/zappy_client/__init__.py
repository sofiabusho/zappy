"""Zappy autonomous AI client (Python).

C01 scope: real ``./client`` CLI, TCP connect, and the subject handshake
(``WELCOME`` → team → ``nb-client`` → ``x y``). C02 adds the command
sender/receiver (:mod:`zappy_client.pipeline`) enforcing the ≤10 pipeline
window (RQ12) and exact subject command syntax (RQ11). C03 adds the autonomous
survival AI (:mod:`zappy_client.survival`, backed by
:mod:`zappy_client.parsing`): it sees, moves, and eats food to avoid starvation
(RQ07) using only the game socket (RQ20). Gathering, rituals and forking land in
C04+; the primitives stay small and testable so those tickets can build on them.
"""

from __future__ import annotations

__all__ = ["__version__"]

__version__ = "0.1.0"

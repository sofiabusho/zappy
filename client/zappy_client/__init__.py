"""Zappy autonomous AI client (Python).

C01 scope: real ``./client`` CLI, TCP connect, and the subject handshake
(``WELCOME`` → team → ``nb-client`` → ``x y``). Gameplay/AI loops land in
later tickets (C02+); this package keeps the connection primitives small and
testable so those tickets can build on them.
"""

from __future__ import annotations

__all__ = ["__version__"]

__version__ = "0.1.0"

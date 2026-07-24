"""Zappy autonomous AI client (Python).

C01 scope: real ``./client`` CLI, TCP connect, and the subject handshake
(``WELCOME`` → team → ``nb-client`` → ``x y``). C02 adds the command
sender/receiver (:mod:`zappy_client.pipeline`) enforcing the ≤10 pipeline
window (RQ12) and exact subject command syntax (RQ11). Survival/AI loops land
in later tickets (C03+); this package keeps the primitives small and testable
so those tickets can build on them.
"""

from __future__ import annotations

__all__ = ["__version__"]

__version__ = "0.1.0"

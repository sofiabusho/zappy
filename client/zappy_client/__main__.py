"""Enable ``python3 -m zappy_client`` as the client entrypoint."""

from __future__ import annotations

from .main import main

if __name__ == "__main__":
    raise SystemExit(main())

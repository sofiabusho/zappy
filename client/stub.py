#!/usr/bin/env python3
"""Client stub (A02): print subject-matching usage until real CLI exists (C01).

Usage strings aligned with docs/raw/audit.md (AQ11) and RQ18 flags.
"""

from __future__ import annotations

import sys

USAGE = """\
Usage: ./client -n <team> -p <port> [-h <hostname>]
 -n team_name
 -p port
 -h name of the host , the default is localhost
"""


def main(argv: list[str] | None = None) -> int:
    # Stub: always print usage. Full arg handling is C01.
    _ = argv if argv is not None else sys.argv[1:]
    sys.stdout.write(USAGE)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

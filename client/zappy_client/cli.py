"""Command-line parsing for ``./client`` (C01 / RQ18, AQ11).

The subject fixes the exact flags and usage block::

    ./client -n <team> -p <port> [-h <hostname>]
      -n team_name
      -p port
      -h name of the host , the default is localhost

We parse by hand rather than with :mod:`argparse` because ``-h`` is the host
flag here, not ``--help``; the subject's usage text must be reproduced
verbatim for the auditor (AQ11).
"""

from __future__ import annotations

from dataclasses import dataclass

DEFAULT_HOST = "localhost"

# Verbatim subject usage (docs/raw/requirements.md "The Client" +
# docs/raw/audit.md client section). Auditors match this exactly (AQ11).
USAGE = (
    "Usage: ./client -n <team> -p <port> [-h <hostname>]\n"
    " -n team_name\n"
    " -p port\n"
    " -h name of the host , the default is localhost\n"
)


class UsageError(Exception):
    """Raised on missing/invalid arguments; caller prints :data:`USAGE`."""


@dataclass(frozen=True)
class ClientArgs:
    """Validated client invocation options."""

    team: str
    port: int
    host: str = DEFAULT_HOST


def _require_value(flag: str, values: list[str], index: int) -> str:
    if index + 1 >= len(values):
        raise UsageError(f"missing value for {flag}")
    return values[index + 1]


def parse_args(argv: list[str]) -> ClientArgs:
    """Parse ``-n``/``-p``/``-h`` flags into :class:`ClientArgs`.

    Raises :class:`UsageError` on anything the subject does not define
    (missing required flags, non-integer/invalid port, unknown flags,
    stray positionals) so the entrypoint can print usage and exit non-zero.
    """
    team: str | None = None
    port_str: str | None = None
    host = DEFAULT_HOST

    i = 0
    while i < len(argv):
        arg = argv[i]
        if arg == "-n":
            team = _require_value("-n", argv, i)
            i += 2
        elif arg == "-p":
            port_str = _require_value("-p", argv, i)
            i += 2
        elif arg == "-h":
            host = _require_value("-h", argv, i)
            i += 2
        else:
            raise UsageError(f"unexpected argument: {arg}")

    if team is None:
        raise UsageError("missing required -n <team>")
    if port_str is None:
        raise UsageError("missing required -p <port>")

    try:
        port = int(port_str)
    except ValueError as exc:
        raise UsageError(f"port must be an integer, got {port_str!r}") from exc
    if not (0 < port < 65536):
        raise UsageError(f"port out of range (1-65535), got {port}")

    if not team:
        raise UsageError("team name must not be empty")

    return ClientArgs(team=team, port=port, host=host)

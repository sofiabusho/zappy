"""CLI parsing + usage tests (C01 / RQ18, AQ11)."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from zappy_client.cli import DEFAULT_HOST, USAGE, UsageError, parse_args

CLIENT_DIR = Path(__file__).resolve().parents[1]
WRAPPER = CLIENT_DIR / "client"

# Verbatim subject usage block (docs/raw/requirements.md + docs/raw/audit.md).
USAGE_LINES = [
    "Usage: ./client -n <team> -p <port> [-h <hostname>]",
    " -n team_name",
    " -p port",
    " -h name of the host , the default is localhost",
]


def test_usage_constant_matches_subject() -> None:
    for line in USAGE_LINES:
        assert line in USAGE


def test_parse_full_args() -> None:
    args = parse_args(["-n", "my_team", "-p", "8080", "-h", "127.0.0.1"])
    assert args.team == "my_team"
    assert args.port == 8080
    assert args.host == "127.0.0.1"


def test_host_defaults_to_localhost() -> None:
    args = parse_args(["-n", "my_team", "-p", "8080"])
    assert args.host == DEFAULT_HOST == "localhost"


def test_flag_order_does_not_matter() -> None:
    args = parse_args(["-p", "4242", "-h", "example", "-n", "team"])
    assert (args.team, args.port, args.host) == ("team", 4242, "example")


@pytest.mark.parametrize(
    "argv",
    [
        [],  # nothing
        ["-n", "team"],  # missing -p
        ["-p", "8080"],  # missing -n
        ["-n", "team", "-p"],  # dangling -p
        ["-n", "team", "-p", "notaport"],  # non-integer port
        ["-n", "team", "-p", "0"],  # out-of-range port
        ["-n", "team", "-p", "70000"],  # out-of-range port
        ["-n", "", "-p", "8080"],  # empty team
        ["-n", "team", "-p", "8080", "extra"],  # stray positional
        ["--nope"],  # unknown flag
    ],
)
def test_bad_args_raise_usage_error(argv: list[str]) -> None:
    with pytest.raises(UsageError):
        parse_args(argv)


def test_wrapper_prints_usage_and_nonzero_without_args() -> None:
    result = subprocess.run(
        [str(WRAPPER)],
        cwd=CLIENT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    for line in USAGE_LINES:
        assert line in result.stdout


def test_module_entrypoint_prints_usage() -> None:
    result = subprocess.run(
        [sys.executable, "-m", "zappy_client", "-n", "team"],  # missing -p
        cwd=CLIENT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "Usage: ./client -n <team> -p <port> [-h <hostname>]" in result.stdout

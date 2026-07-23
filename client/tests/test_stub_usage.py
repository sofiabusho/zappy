"""Tests for the A02 client usage stub (wired into pytest in A03)."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

CLIENT_DIR = Path(__file__).resolve().parents[1]
STUB = CLIENT_DIR / "stub.py"

USAGE_LINES = [
    "Usage: ./client -n <team> -p <port> [-h <hostname>]",
    " -n team_name",
    " -p port",
    " -h name of the host , the default is localhost",
]


def test_stub_prints_subject_usage() -> None:
    result = subprocess.run(
        [sys.executable, str(STUB)],
        cwd=CLIENT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    for line in USAGE_LINES:
        assert line in result.stdout

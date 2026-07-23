#!/usr/bin/env python3
"""A02 wrapper stub checks: usage strings match audit subject (RQ17, RQ18, AQ01, AQ11)."""

from __future__ import annotations

import os
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SERVER_WRAPPER = ROOT / "server" / "server"
CLIENT_WRAPPER = ROOT / "client" / "client"

# Exact lines from docs/raw/audit.md (AQ01 / AQ11 samples).
SERVER_USAGE_LINES = [
    " Usage: ./server -p <port> -x <width> -y <height> -n <team> [<team>] [<team>] ... -c <nb> [-t <t>]",
    " -p port number",
    " -x world width",
    " -y world height",
    " -n team_name_1 team_name_2 ...",
    " -c number of clients authorized at the beginning of the game",
    " -t [100] time unit divider (the greater t is, the faster the game will go)",
]

CLIENT_USAGE_LINES = [
    "Usage: ./client -n <team> -p <port> [-h <hostname>]",
    " -n team_name",
    " -p port",
    " -h name of the host , the default is localhost",
]


def run_wrapper(path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(path)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


class WrapperStubTests(unittest.TestCase):
    def test_server_wrapper_executable(self) -> None:
        self.assertTrue(SERVER_WRAPPER.is_file(), "server/server missing")
        self.assertTrue(os.access(SERVER_WRAPPER, os.X_OK), "server/server not executable")

    def test_client_wrapper_executable(self) -> None:
        self.assertTrue(CLIENT_WRAPPER.is_file(), "client/client missing")
        self.assertTrue(os.access(CLIENT_WRAPPER, os.X_OK), "client/client not executable")

    def test_server_usage_matches_audit(self) -> None:
        """AQ01 / RQ17: ./server prints subject-like usage with required flags."""
        result = run_wrapper(SERVER_WRAPPER)
        self.assertNotEqual(result.returncode, 0)
        for line in SERVER_USAGE_LINES:
            self.assertIn(line, result.stdout)
        for flag in ("-p", "-x", "-y", "-n", "-c", "-t"):
            self.assertIn(flag, result.stdout)

    def test_client_usage_matches_audit(self) -> None:
        """AQ11 / RQ18: ./client prints subject-like usage with required flags."""
        result = run_wrapper(CLIENT_WRAPPER)
        self.assertNotEqual(result.returncode, 0)
        for line in CLIENT_USAGE_LINES:
            self.assertIn(line, result.stdout)
        for flag in ("-n", "-p", "-h"):
            self.assertIn(flag, result.stdout)
        self.assertIn("localhost", result.stdout)


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)

"""A03 harness checks: scripts/check.sh exists and is executable."""

from __future__ import annotations

import os
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts" / "check.sh"


class CheckHarnessTests(unittest.TestCase):
    def test_check_sh_exists_and_executable(self) -> None:
        self.assertTrue(CHECK.is_file(), "scripts/check.sh missing")
        self.assertTrue(os.access(CHECK, os.X_OK), "scripts/check.sh not executable")

    def test_check_sh_mentions_toolchains(self) -> None:
        text = CHECK.read_text(encoding="utf-8")
        for needle in ("cargo", "ruff", "pytest", "npm run lint"):
            self.assertIn(needle, text)


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)

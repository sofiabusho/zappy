#!/usr/bin/env python3
"""A04 README quickstart checks: build/run/audit docs + RQ16 + siege warning."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"


class ReadmeQuickstartTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.text = README.read_text(encoding="utf-8")

    def test_readme_exists(self) -> None:
        self.assertTrue(README.is_file())

    def test_localhost_only(self) -> None:
        lower = self.text.lower()
        self.assertIn("localhost", lower)
        self.assertIn("127.0.0.1", self.text)

    def test_siege_warning(self) -> None:
        lower = self.text.lower()
        self.assertIn("siege", lower)
        self.assertIn("warning", lower)
        # Subject: only own server; never without permission
        self.assertTrue(
            "own" in lower and ("never" in lower or "do **never**" in lower),
            "siege warning should forbid use on others' servers",
        )

    def test_rq16_constraints_documented(self) -> None:
        """RQ16: Rust (C/C++/Rust), multiplexed TCP, no exec*, never hang forever."""
        text = self.text
        lower = text.lower()
        self.assertIn("Rust", text)
        self.assertTrue("multiplex" in lower or "multiplexing" in lower)
        self.assertIn("exec", lower)
        self.assertTrue(
            "hang" in lower or "never block" in lower,
            "must document never-hang / non-blocking availability",
        )

    def test_build_and_check_commands(self) -> None:
        self.assertIn("scripts/check.sh", self.text)
        self.assertIn("cargo", self.text)
        self.assertIn("ruff", self.text)
        self.assertIn("npm", self.text)


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)

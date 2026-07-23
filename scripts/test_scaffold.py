#!/usr/bin/env python3
"""A01 scaffold checks: required dirs exist; root README points at AGENTS.md (RQ01)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_DIRS = ("server", "client", "gui", "scripts")


class ScaffoldTests(unittest.TestCase):
    def test_required_directories_exist(self) -> None:
        missing = [name for name in REQUIRED_DIRS if not (ROOT / name).is_dir()]
        self.assertEqual(missing, [], f"missing directories: {missing}")

    def test_readme_points_at_agents(self) -> None:
        readme = ROOT / "README.md"
        self.assertTrue(readme.is_file(), "root README.md missing")
        text = readme.read_text(encoding="utf-8")
        self.assertIn("AGENTS.md", text)

    def test_rq01_parts_documented(self) -> None:
        """RQ01: system has server, AI client(s), and graphic client."""
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        for part in ("server/", "client/", "gui/"):
            self.assertIn(part, readme)
        for name in REQUIRED_DIRS:
            self.assertTrue((ROOT / name / "README.md").is_file(), f"{name}/README.md missing")


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)

#!/usr/bin/env python3
# Purpose: minimal unit tests for scripts helper modules.
# Inputs: package path/reporting helpers.
# Outputs: unittest pass/fail for helper contracts.
from __future__ import annotations

import unittest
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "packages" / "bijux-atlas-scripts" / "src"))

from bijux_atlas_scripts.core.paths import find_repo_root
from bijux_atlas_scripts.reporting import script_output_dir


class PathsTests(unittest.TestCase):
    def test_repo_root_has_makefile(self) -> None:
        root = find_repo_root()
        self.assertTrue((root / "Makefile").exists())

    def test_artifacts_scripts_dir_shape(self) -> None:
        out = script_output_dir("example", "run-1")
        self.assertEqual(Path("artifacts/scripts/example/run-1"), out.relative_to(find_repo_root()))

    def test_reporting_output_dir(self) -> None:
        out = script_output_dir("unit-test", "run-2")
        self.assertTrue(out.exists())
        self.assertIn("artifacts/scripts/unit-test/run-2", out.as_posix())


if __name__ == "__main__":
    unittest.main()

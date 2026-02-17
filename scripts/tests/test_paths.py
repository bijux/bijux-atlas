#!/usr/bin/env python3
# Purpose: minimal unit tests for scripts helper modules.
# Inputs: scripts.internal.paths and scripts.tools modules.
# Outputs: unittest pass/fail for helper contracts.
from __future__ import annotations

import unittest
from pathlib import Path

from scripts.internal import paths
from scripts.tools import reporting


class PathsTests(unittest.TestCase):
    def test_repo_root_has_makefile(self) -> None:
        root = paths.repo_root()
        self.assertTrue((root / "Makefile").exists())

    def test_artifacts_scripts_dir_shape(self) -> None:
        out = paths.artifacts_scripts_dir("example", "run-1")
        self.assertEqual(Path("artifacts/scripts/example/run-1"), out.relative_to(paths.repo_root()))

    def test_reporting_output_dir(self) -> None:
        out = reporting.script_output_dir("unit-test", "run-2")
        self.assertTrue(out.exists())
        self.assertIn("artifacts/scripts/unit-test/run-2", out.as_posix())


if __name__ == "__main__":
    unittest.main()

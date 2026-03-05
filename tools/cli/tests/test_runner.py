from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from tools.cli.atlas_cli_runner import apply_env_overrides, build_command, load_config, validate_config


class RunnerTests(unittest.TestCase):
    def test_load_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "config.toml"
            path.write_text('repo_root = "/tmp/repo"\noutput_format = "json"\nquiet = true\n', encoding="utf-8")
            cfg = load_config(path)
            self.assertEqual(cfg["repo_root"], "/tmp/repo")

    def test_validate_rejects_unknown(self) -> None:
        with self.assertRaises(ValueError):
            validate_config({"unknown": 1})

    def test_build_command(self) -> None:
        cmd = build_command("bin", {"repo_root": "/tmp/repo", "output_format": "json", "quiet": True}, ["api", "list"])
        self.assertEqual(cmd[:7], ["bin", "--repo-root", "/tmp/repo", "--output-format", "json", "--quiet", "api"])

    def test_apply_env_overrides(self) -> None:
        import os

        os.environ["BIJUX_DEV_ATLAS_OUTPUT_FORMAT"] = "both"
        out = apply_env_overrides({})
        self.assertEqual(out["output_format"], "both")
        del os.environ["BIJUX_DEV_ATLAS_OUTPUT_FORMAT"]


if __name__ == "__main__":
    unittest.main()

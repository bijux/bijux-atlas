from __future__ import annotations

import pathlib
import unittest

from tools.cli.discover_subcommands import parse_help_commands


class HelpRegressionTests(unittest.TestCase):
    def test_help_snapshot_has_required_commands(self) -> None:
        text = pathlib.Path("tools/cli/fixtures/help-root.snapshot.txt").read_text(encoding="utf-8")
        commands = parse_help_commands(text)
        for required in ["api", "observe", "ops", "check", "tests"]:
            self.assertIn(required, commands)


if __name__ == "__main__":
    unittest.main()

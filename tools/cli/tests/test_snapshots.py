from __future__ import annotations

import json
import pathlib
import unittest

from tools.cli.discover_subcommands import GROUPS, parse_help_commands


class SnapshotTests(unittest.TestCase):
    def test_discovery_snapshot(self) -> None:
        fixtures = pathlib.Path("tools/cli/fixtures")
        help_text = (fixtures / "help-root.snapshot.txt").read_text(encoding="utf-8")
        expected = json.loads((fixtures / "discovery.snapshot.json").read_text(encoding="utf-8"))

        grouped: dict[str, list[str]] = {}
        for cmd in parse_help_commands(help_text):
            key = GROUPS.get(cmd, "misc")
            grouped.setdefault(key, []).append(cmd)
        grouped = {k: sorted(v) for k, v in grouped.items()}

        self.assertEqual(grouped, expected)


if __name__ == "__main__":
    unittest.main()

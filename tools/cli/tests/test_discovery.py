from __future__ import annotations

import unittest

from tools.cli.discover_subcommands import parse_help_commands


HELP_TEXT = """
Usage: bijux-dev-atlas [OPTIONS] [COMMAND]

Commands:
  api       API contract operations
  observe   Observability commands
  load      Load testing commands
  help      Print this message

Options:
  -h, --help  Print help
"""


class DiscoveryTests(unittest.TestCase):
    def test_parse_commands(self) -> None:
        self.assertEqual(parse_help_commands(HELP_TEXT), ["api", "observe", "load"])


if __name__ == "__main__":
    unittest.main()

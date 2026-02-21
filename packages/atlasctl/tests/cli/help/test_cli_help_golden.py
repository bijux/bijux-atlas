from __future__ import annotations

import json

from tests.helpers import golden_text, run_atlasctl


def test_help_json_command_names_match_golden() -> None:
    proc = run_atlasctl("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = [entry["name"] for entry in payload["commands"]]
    golden = golden_text("cli_help_commands.expected.txt")
    expected = [line.strip() for line in golden.splitlines() if line.strip()]
    assert names == expected

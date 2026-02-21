from __future__ import annotations

import json

from helpers import run_atlasctl


def test_gen_goldens_writes_expected_targets() -> None:
    proc = run_atlasctl("--quiet", "gen", "goldens")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"
    expected = {
        "help.json.golden",
        "commands.json.golden",
        "surface.json.golden",
        "explain.check.json.golden",
        "check-list.json.golden",
        "cli_help_snapshot.txt",
        "cli_help_commands.expected.txt",
    }
    assert expected.issubset(payload["written"].keys())

from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_help_output_hides_internal_commands_by_default() -> None:
    proc = run_atlasctl("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {entry["name"] for entry in payload["commands"]}
    assert "internal" not in names


def test_help_output_include_internal_shows_internal_group() -> None:
    proc = run_atlasctl("--quiet", "help", "--include-internal", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {entry["name"] for entry in payload["commands"]}
    assert "internal" in names

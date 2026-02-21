from __future__ import annotations

import json

from helpers import run_atlasctl


def test_help_output_hides_legacy_and_compat_by_default() -> None:
    proc = run_atlasctl("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {entry["name"] for entry in payload["commands"]}
    assert "legacy" not in names
    assert "compat" not in names


def test_help_output_include_internal_shows_legacy_and_compat() -> None:
    proc = run_atlasctl("--quiet", "help", "--include-internal", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {entry["name"] for entry in payload["commands"]}
    assert "legacy" in names
    assert "compat" in names

from __future__ import annotations

import json

from tests.helpers import golden_text, run_atlasctl


def _golden(name: str) -> str:
    return golden_text(name)


def test_list_commands_hides_internal_by_default() -> None:
    proc = run_atlasctl("--quiet", "list", "commands", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {item["name"] for item in payload["items"]}
    assert "internal" not in names


def test_list_commands_include_internal_flag() -> None:
    proc = run_atlasctl("--quiet", "list", "commands", "--json", "--include-internal")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {item["name"] for item in payload["items"]}
    assert "internal" in names


def test_registry_inventory_is_stable() -> None:
    checks = run_atlasctl("--quiet", "list", "checks", "--json")
    commands = run_atlasctl("--quiet", "list", "commands", "--json")
    suites = run_atlasctl("--quiet", "list", "suites", "--json")
    assert checks.returncode == 0, checks.stderr
    assert commands.returncode == 0, commands.stderr
    assert suites.returncode == 0, suites.stderr
    assert checks.stdout.strip() == _golden("list-checks.json.golden")
    assert commands.stdout.strip() == _golden("list-commands.json.golden")
    assert suites.stdout.strip() == _golden("list-suites.json.golden")

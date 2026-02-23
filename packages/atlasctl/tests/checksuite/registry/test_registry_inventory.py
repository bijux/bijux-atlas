from __future__ import annotations

import json

from atlasctl.contracts.validate import validate
from tests.helpers import golden_text, run_atlasctl


def _golden(name: str) -> str:
    return golden_text(name)


def test_list_commands_hides_internal_by_default() -> None:
    proc = run_atlasctl("--quiet", "--format", "json", "list", "commands")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {item["name"] for item in payload["items"]}
    assert "internal" not in names


def test_list_commands_include_internal_flag() -> None:
    proc = run_atlasctl("--quiet", "--format", "json", "list", "commands", "--include-internal")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {item["name"] for item in payload["items"]}
    assert "internal" in names


def test_registry_inventory_is_stable() -> None:
    checks = run_atlasctl("--quiet", "--format", "json", "list", "checks")
    commands = run_atlasctl("--quiet", "--format", "json", "list", "commands")
    suites = run_atlasctl("--quiet", "--format", "json", "list", "suites")
    assert checks.returncode == 0, checks.stderr
    assert commands.returncode == 0, commands.stderr
    assert suites.returncode == 0, suites.stderr
    checks_payload = json.loads(checks.stdout)
    commands_payload = json.loads(commands.stdout)
    suites_payload = json.loads(suites.stdout)
    validate(checks_payload["schema_name"], checks_payload)
    validate(commands_payload["schema_name"], commands_payload)
    validate(suites_payload["schema_name"], suites_payload)
    assert checks.stdout.strip() == _golden("list-checks.json.golden")
    assert commands.stdout.strip() == _golden("list-commands.json.golden")
    assert suites.stdout.strip() == _golden("list-suites.json.golden")


def test_list_goldens_validate_new_list_schemas() -> None:
    for name, schema in (
        ("list-checks.json.golden", "atlasctl.list-checks.v1"),
        ("list-commands.json.golden", "atlasctl.list-commands.v1"),
        ("list-suites.json.golden", "atlasctl.list-suites.v1"),
    ):
        payload = json.loads(_golden(name))
        validate(schema, payload)

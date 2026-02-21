from __future__ import annotations

import json
from pathlib import Path

from atlasctl.contracts.validate import validate
from atlasctl.suite.command import load_suites
from tests.helpers import ROOT, golden_text, run_atlasctl


def _golden(name: str) -> str:
    return golden_text(name)


def _normalize(payload: dict[str, object]) -> dict[str, object]:
    if "run_id" in payload:
        payload = dict(payload)
        payload["run_id"] = ""
    return payload


def test_every_public_command_accepts_global_json_flag() -> None:
    help_proc = run_atlasctl("--quiet", "help", "--json")
    assert help_proc.returncode == 0, help_proc.stderr
    commands = json.loads(help_proc.stdout)["commands"]
    for command in commands:
        name = str(command["name"])
        proc = run_atlasctl("--json", name, "--help")
        assert proc.returncode == 0, f"{name}: {proc.stderr}"


def test_json_outputs_validate_against_declared_schema() -> None:
    commands = [
        ("help", "--json"),
        ("commands", "--json"),
        ("surface", "--json"),
        ("--json", "check", "list"),
        ("explain", "command", "check", "--json"),
        ("--json", "contracts", "list"),
    ]
    for args in commands:
        proc = run_atlasctl("--quiet", *args)
        assert proc.returncode == 0, (args, proc.stderr)
        payload = json.loads(proc.stdout)
        schema_name = payload.get("schema_name")
        assert isinstance(schema_name, str), f"{args}: missing schema_name"
        validate(schema_name, payload)


def test_group_representative_outputs_golden() -> None:
    groups = ["docs", "configs", "dev", "ops", "policies", "internal"]
    payload: dict[str, object] = {}
    for group in groups:
        proc = run_atlasctl("--quiet", "--json", "explain", group)
        assert proc.returncode == 0, (group, proc.stderr)
        parsed = json.loads(proc.stdout)
        validate(parsed["schema_name"], parsed)
        payload[group] = _normalize(parsed)
    assert json.dumps(payload, sort_keys=True) == _golden("group-output-representatives.json.golden")


def test_refgrade_suite_has_contracts_validate_gate() -> None:
    _, suites = load_suites(Path(__file__).resolve().parents[4])
    refgrade = suites["refgrade"]
    assert "cmd:atlasctl contracts validate --report json" in refgrade.items

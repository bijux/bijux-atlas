from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_ops_suite_run_json_schema_contract_dry_run() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "ops", "--dry-run", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["tool"] == "atlasctl"
    assert payload["status"] == "ok"
    assert payload["command"] == "suite"
    assert payload["dry_run"] is True
    assert "run_id" in payload


def test_suite_list_json_shape_contract() -> None:
    proc = run_atlasctl("--quiet", "suite", "list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["tool"] == "atlasctl"
    assert payload["status"] == "ok"
    assert isinstance(payload["first_class_suites"], list)
    assert isinstance(payload["suites"], list)
    for row in payload["first_class_suites"][:5]:
        assert "name" in row and isinstance(row["name"], str)
        assert "check_count" in row

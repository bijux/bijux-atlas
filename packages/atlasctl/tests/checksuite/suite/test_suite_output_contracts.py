from __future__ import annotations

import json

from atlasctl.contracts.validate import validate
from tests.helpers import golden_text, run_atlasctl


def _golden(name: str) -> str:
    return golden_text(name)


def test_suite_required_list_output_golden() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "required", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("suite_required.expected.txt")


def test_suite_run_json_schema_contract(tmp_path) -> None:
    target = tmp_path / "suite-run"
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check checks_repo_module_size", "--json", "--target-dir", str(target))
    assert proc.returncode in {0, 1, 2}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_name"] == "atlasctl.suite-run.v1"
    validate("atlasctl.suite-run.v1", payload)

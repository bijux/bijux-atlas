from __future__ import annotations

import json

from tests.helpers import golden_text, run_atlasctl


def test_contracts_list_json_matches_golden() -> None:
    # schema-validate-exempt: contracts list payload does not have a dedicated schema yet.
    proc = run_atlasctl("--quiet", "contracts", "list", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    golden = golden_text("contracts-list.json.golden")
    assert json.loads(proc.stdout) == json.loads(golden)


def test_contracts_lint_json_passes() -> None:
    proc = run_atlasctl("--quiet", "contracts", "lint", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"


def test_contracts_validate_self_json_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "contracts", "validate-self", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    payload["run_id"] = "pytest-run"
    golden = json.loads(golden_text("contracts-validate-self.json.golden"))
    assert payload == golden

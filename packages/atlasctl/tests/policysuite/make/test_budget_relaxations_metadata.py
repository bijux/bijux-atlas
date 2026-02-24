from __future__ import annotations

import json
from pathlib import Path

from atlasctl.checks.tools.policies_domain.make import check_policies_budget_relaxations_expiry_and_issue


def test_budget_relaxations_issue_and_expiry_pass(tmp_path: Path) -> None:
    path = tmp_path / "configs/policy/budget-relaxations.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "exceptions": [
                    {"id": "sample", "issue": "ISSUE-100", "expiry": "2099-01-01"},
                ],
            },
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    code, errors = check_policies_budget_relaxations_expiry_and_issue(tmp_path)
    assert code == 0
    assert errors == []


def test_budget_relaxations_issue_and_expiry_fail(tmp_path: Path) -> None:
    path = tmp_path / "configs/policy/budget-relaxations.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "exceptions": [
                    {"id": "sample", "issue": "", "expiry": "2020-01-01"},
                ],
            },
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    code, errors = check_policies_budget_relaxations_expiry_and_issue(tmp_path)
    assert code == 1
    assert any("invalid or missing issue" in item for item in errors)
    assert any("expired on 2020-01-01" in item for item in errors)

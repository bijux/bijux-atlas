from __future__ import annotations

# BYPASS_TEST_OK: budget report fixtures intentionally use configs/policy paths.
# schema-validate-exempt: report_budgets helper payload has no dedicated schema contract yet.
import json
from pathlib import Path

from atlasctl.commands.policies.runtime.dir_entry_budgets import report_budgets
from tests.helpers import golden_text


def _write_exceptions(repo: Path) -> None:
    path = repo / "configs/policy/BUDGET_EXCEPTIONS.yml"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text('{"schema_version":1,"max_exceptions":3,"exceptions":[]}', encoding="utf-8")


def _build_fixture(repo: Path) -> None:
    d1 = repo / "packages/atlasctl/src/atlasctl/docs"
    d1.mkdir(parents=True)
    for idx in range(11):
        (d1 / f"m{idx}.py").write_text("x=1\n", encoding="utf-8")
    d2 = repo / "packages/atlasctl/tests/docs"
    d2.mkdir(parents=True)
    for idx in range(9):
        (d2 / f"t{idx}.py").write_text("x=1\n", encoding="utf-8")


def test_budget_report_by_domain_golden(tmp_path: Path) -> None:
    _build_fixture(tmp_path)
    _write_exceptions(tmp_path)
    payload = report_budgets(tmp_path, by_domain=True)
    got = json.dumps(payload, sort_keys=True)
    assert got == golden_text("report_budgets_by_domain.json.golden")

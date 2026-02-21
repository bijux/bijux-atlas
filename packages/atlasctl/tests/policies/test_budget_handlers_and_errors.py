from __future__ import annotations

import argparse
from pathlib import Path

from atlasctl.commands.policies.runtime import budget_handlers
from atlasctl.commands.policies.runtime.errors import BudgetMetricError


def test_budget_metric_error_has_stable_code() -> None:
    err = BudgetMetricError("bad metric")
    assert err.code == "POL001"
    assert err.message == "bad metric"


def test_handle_budget_command_returns_none_for_non_budget_cmd(tmp_path: Path) -> None:
    ns = argparse.Namespace(policies_cmd="check", report="json")
    code = budget_handlers.handle_budget_command(ns, tmp_path, lambda _repo, _out_file, _content: None)
    assert code is None

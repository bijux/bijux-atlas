from __future__ import annotations

from .engine import (
    BudgetProfile,
    EngineOptions,
    extract_failures,
    group_failures_by_domain_area,
    run_checks as _engine_run_checks,
    stable_exit_code,
    top_n_slowest,
)
from pathlib import Path

from .model import CheckContext, CheckRunReport, CheckSelector
from .policy import Capabilities
from .report import report_from_payload


def run_checks(
    ctx: CheckContext | Path,
    selections: CheckSelector | None = None,
    fail_fast: bool = False,
    budget_profile: BudgetProfile | None = None,
    capabilities: Capabilities | None = None,
) -> CheckRunReport:
    opts = EngineOptions(fail_fast=bool(fail_fast), only_fast=False, include_slow=True, include_internal=True)
    return _engine_run_checks(
        None,
        selections,
        ctx,
        options=opts,
        budget_profile=budget_profile,
        capabilities=capabilities,
    )

__all__ = [
    "BudgetProfile",
    "EngineOptions",
    "extract_failures",
    "group_failures_by_domain_area",
    "report_from_payload",
    "run_checks",
    "stable_exit_code",
    "top_n_slowest",
]

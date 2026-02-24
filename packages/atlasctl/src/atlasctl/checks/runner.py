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

from .model import CheckContext, CheckResult, CheckRunReport, CheckSelector
from .policy import Capabilities


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


def report_from_payload(payload: dict[str, object]) -> CheckRunReport:
    raw_rows = payload.get("rows", [])
    rows: list[CheckResult] = []
    for row in raw_rows if isinstance(raw_rows, list) else []:
        if not isinstance(row, dict):
            continue
        rows.append(
            CheckResult(
                id=str(row.get("id", "")),
                title=str(row.get("title", row.get("id", ""))),
                domain=str(row.get("domain", "")),
                status=str(row.get("status", "")).lower(),
                errors=tuple(
                    str(row.get("detail", row.get("reason", ""))).strip()
                    for _ in [0]
                    if str(row.get("detail", row.get("reason", ""))).strip()
                ),
                metrics={"duration_ms": int(row.get("duration_ms", 0))},
                result_code=str(row.get("result_code", "CHECK_GENERIC")),
            )
        )
    summary = payload.get("summary", {})
    timings = {"duration_ms": int(summary.get("duration_ms", 0))} if isinstance(summary, dict) else {}
    return CheckRunReport(rows=tuple(rows), timings=timings)


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

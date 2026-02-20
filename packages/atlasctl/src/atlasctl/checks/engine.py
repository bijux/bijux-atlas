from __future__ import annotations

import time
from pathlib import Path

from .base import CheckDef, CheckResult


class CommandCheckDef:
    def __init__(self, check_id: str, domain: str, cmd: list[str], budget_ms: int = 10_000) -> None:
        self.check_id = check_id
        self.domain = domain
        self.cmd = cmd
        self.budget_ms = budget_ms


def run_function_checks(repo_root: Path, checks: list[CheckDef]) -> tuple[int, list[CheckResult]]:
    rows: list[CheckResult] = []
    failed = 0
    for chk in checks:
        start = time.perf_counter()
        try:
            code, errors = chk.fn(repo_root)
        except Exception as exc:
            code, errors = 1, [f"internal check error: {exc}"]
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        status = "pass" if code == 0 else "fail"
        if code != 0:
            failed += 1
        rows.append(
            CheckResult(
                check_id=chk.check_id,
                domain=chk.domain,
                status=status,
                duration_ms=elapsed_ms,
                budget_ms=chk.budget_ms,
                budget_status="pass" if elapsed_ms <= chk.budget_ms else "warn",
                errors=errors,
                severity=chk.severity.value,
                evidence=chk.evidence,
            )
        )
    return failed, rows

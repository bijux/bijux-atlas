from __future__ import annotations

# Canonical check engine execution runtime (migrated from `checks/core/execution.py`
# during Phase 3). Keep dependencies narrow and move policy enforcement to
# runtime/capability boundaries over time.

import time
import signal
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Callable

from ..core.base import CheckDef, CheckResult
from ..registry.catalog import check_tags
from ...core.process import run_command


class CommandCheckDef:
    def __init__(self, check_id: str, domain: str, cmd: list[str], budget_ms: int = 10_000) -> None:
        self.check_id = check_id
        self.domain = domain
        self.cmd = cmd
        self.budget_ms = budget_ms


def run_command_checks(repo_root: Path, checks: list[CommandCheckDef]) -> tuple[int, list[dict[str, object]]]:
    rows: list[dict[str, object]] = []
    failed = 0
    for chk in checks:
        start = time.perf_counter()
        result = run_command(chk.cmd, repo_root)
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        status = "pass" if result.code == 0 else "fail"
        if result.code != 0:
            failed += 1
        row: dict[str, object] = {
            "id": chk.check_id,
            "domain": chk.domain,
            "command": " ".join(chk.cmd),
            "status": status,
            "duration_ms": elapsed_ms,
            "budget_ms": chk.budget_ms,
            "budget_status": "pass" if elapsed_ms <= chk.budget_ms else "warn",
        }
        if result.code != 0:
            row["error"] = result.combined_output
        rows.append(row)
    return failed, rows


def run_function_checks(
    repo_root: Path,
    checks: list[CheckDef],
    on_result: Callable[[CheckResult], None] | None = None,
    timeout_ms: int | None = None,
    jobs: int = 1,
) -> tuple[int, list[CheckResult]]:
    checks = sorted(checks, key=lambda c: c.check_id)

    def _run_one(chk: CheckDef) -> CheckResult:
        start = time.perf_counter()
        try:
            if timeout_ms and timeout_ms > 0 and jobs <= 1:
                class _CheckTimeoutError(Exception):
                    pass

                def _raise_timeout(_signum: int, _frame: object) -> None:
                    raise _CheckTimeoutError()

                prev_handler = signal.getsignal(signal.SIGALRM)
                signal.signal(signal.SIGALRM, _raise_timeout)
                signal.setitimer(signal.ITIMER_REAL, timeout_ms / 1000.0)
                try:
                    code, errors = chk.fn(repo_root)
                except _CheckTimeoutError:
                    code, errors = 1, [f"check timed out after {timeout_ms}ms"]
                finally:
                    signal.setitimer(signal.ITIMER_REAL, 0)
                    signal.signal(signal.SIGALRM, prev_handler)
            else:
                code, errors = chk.fn(repo_root)
        except Exception as exc:
            code, errors = 1, [f"internal check error: {exc}"]
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        warnings = sorted([msg.removeprefix("WARN: ").strip() for msg in errors if msg.startswith("WARN:")])
        normalized_errors = sorted([msg for msg in errors if not msg.startswith("WARN:")])
        status = "pass" if code == 0 else "fail"
        return CheckResult(
            id=chk.check_id,
            title=chk.title,
            domain=chk.domain,
            status=status,
            errors=normalized_errors,
            warnings=warnings,
            evidence_paths=list(chk.evidence),
            metrics={
                "duration_ms": elapsed_ms,
                "budget_ms": chk.budget_ms,
                "budget_status": "pass" if elapsed_ms <= chk.budget_ms else "warn",
            },
            description=chk.description,
            fix_hint=chk.fix_hint,
            category=chk.category.value,
            severity=chk.severity.value,
            tags=list(check_tags(chk)),
            effects=list(chk.effects),
            owners=list(chk.owners),
            writes_allowed_roots=list(chk.writes_allowed_roots),
        )

    rows: list[CheckResult] = []
    failed = 0
    if jobs > 1:
        with ThreadPoolExecutor(max_workers=jobs) as ex:
            produced = list(ex.map(_run_one, checks))
        rows.extend(produced)
    else:
        for chk in checks:
            rows.append(_run_one(chk))
    for row in rows:
        if row.status != "pass":
            failed += 1
        if on_result is not None:
            on_result(row)
    return failed, rows

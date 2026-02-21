from __future__ import annotations

import time
from pathlib import Path

from .base import CheckDef, CheckResult
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
        warnings = sorted([msg.removeprefix("WARN: ").strip() for msg in errors if msg.startswith("WARN:")])
        normalized_errors = sorted([msg for msg in errors if not msg.startswith("WARN:")])
        status = "pass" if code == 0 else "fail"
        if code != 0:
            failed += 1
        rows.append(
            CheckResult(
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
        )
    return failed, rows

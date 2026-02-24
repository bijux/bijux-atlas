from __future__ import annotations

import builtins
import inspect
import os
import time
from concurrent.futures import ThreadPoolExecutor, TimeoutError as FutureTimeoutError
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .evidence import DEFAULT_WRITE_ROOT, ensure_explicit_repo_root, validate_evidence_paths
from .model import (
    CheckContext,
    CheckDef,
    CheckResult,
    CheckRunReport,
    CheckSelector,
    CheckStatus,
    Effect,
    Severity,
    Violation,
)
from .policy import Capabilities, EffectPolicy, evaluate_effects
from .registry import list_checks
from .selectors import select_checks


@dataclass(frozen=True)
class BudgetProfile:
    default_timeout_ms: int = 2_000
    timeout_cap_ms: int = 30_000

    def timeout_for(self, check: CheckDef) -> int:
        budget = int(getattr(check, "budget_ms", 0) or 0)
        effective = budget if budget > 0 else self.default_timeout_ms
        return min(effective, self.timeout_cap_ms)


def _run_with_timeout(fn: Any, arg: Any, timeout_ms: int) -> Any:
    with ThreadPoolExecutor(max_workers=1) as pool:
        future = pool.submit(fn, arg)
        return future.result(timeout=max(0.001, timeout_ms / 1000.0))


def _call_check_function(check: CheckDef, ctx: CheckContext) -> tuple[int, list[str], tuple[Violation, ...]]:
    signature = inspect.signature(check.fn)
    first = next(iter(signature.parameters.values()), None)
    argument = ctx.repo_root
    if first is not None and str(first.annotation).endswith("CheckContext"):
        argument = ctx
    result = check.fn(argument)
    if isinstance(result, list) and all(isinstance(item, Violation) for item in result):
        violations = tuple(sorted(result, key=lambda item: item.canonical_key))
        has_error = any(item.severity != Severity.WARN for item in violations)
        warnings = sorted(item.message for item in violations if item.severity == Severity.WARN)
        return (1 if has_error else 0), warnings, violations
    if isinstance(result, tuple) and len(result) == 2:
        code, messages = result
        normalized = sorted(str(item) for item in messages)
        return int(code), normalized, ()
    return (0, [], ())


def _speed_label(check: CheckDef, duration_ms: int) -> str:
    if bool(getattr(check, "slow", False)):
        return "slow"
    return "slow" if int(duration_ms) >= 1000 else "fast"


def _make_result(
    check: CheckDef,
    *,
    status: CheckStatus,
    duration_ms: int,
    messages: list[str],
    violations: tuple[Violation, ...] = (),
    warnings: list[str] | None = None,
    effects_used: tuple[str, ...] = (Effect.FS_READ.value,),
) -> CheckResult:
    if not violations:
        stable_messages = sorted(str(msg) for msg in messages)
        violations = tuple(
            Violation(code=str(check.result_code), message=msg, hint=str(check.fix_hint), severity=Severity.ERROR)
            for msg in stable_messages
        )
    return CheckResult(
        id=str(check.id),
        title=str(check.title),
        domain=str(check.domain),
        status=status,
        violations=violations,
        warnings=tuple(warnings or ()),
        evidence_paths=tuple(check.evidence),
        metrics={
            "duration_ms": int(duration_ms),
            "budget_ms": int(check.budget_ms),
            "speed": _speed_label(check, duration_ms),
        },
        description=str(check.description),
        fix_hint=str(check.fix_hint),
        category=getattr(check.category, "value", str(check.category)),
        severity=check.severity,
        tags=tuple(check.tags),
        effects=tuple(check.effects),
        effects_used=effects_used,
        owners=tuple(check.owners),
        writes_allowed_roots=tuple(check.writes_allowed_roots),
        result_code=str(check.result_code),
    )


def _run_single_check(
    check: CheckDef,
    ctx: CheckContext,
    *,
    effect_policy: EffectPolicy,
    budget_profile: BudgetProfile,
) -> CheckResult:
    allow_effects, reasons = evaluate_effects(check, effect_policy)
    if not allow_effects:
        return _make_result(
            check,
            status=CheckStatus.SKIP,
            duration_ms=0,
            messages=[],
            warnings=[f"skipped by effect policy: {reason}" for reason in reasons],
            effects_used=(Effect.FS_READ.value,),
        )
    ok, evidence_errors = validate_evidence_paths(
        ctx.repo_root,
        tuple(check.evidence),
        allowed_roots=tuple(check.writes_allowed_roots) or (DEFAULT_WRITE_ROOT,),
    )
    if not ok:
        return _make_result(
            check,
            status=CheckStatus.ERROR,
            duration_ms=0,
            messages=[f"invalid evidence path: {item}" for item in evidence_errors],
            effects_used=(Effect.FS_READ.value,),
        )

    printed: list[str] = []
    original_print = builtins.print
    if effect_policy.forbid_print:
        builtins.print = lambda *args, **kwargs: printed.append(" ".join(str(item) for item in args))  # type: ignore[assignment]

    started = time.perf_counter()
    timeout_ms = budget_profile.timeout_for(check)
    try:
        code, errors, structured = _run_with_timeout(lambda chk: _call_check_function(chk, ctx), check, timeout_ms)
        duration_ms = int((time.perf_counter() - started) * 1000)
    except FutureTimeoutError:
        duration_ms = int((time.perf_counter() - started) * 1000)
        return _make_result(
            check,
            status=CheckStatus.ERROR,
            duration_ms=duration_ms,
            messages=[f"timeout after {timeout_ms}ms"],
            effects_used=tuple(check.effects),
        )
    except Exception as exc:  # pragma: no cover
        duration_ms = int((time.perf_counter() - started) * 1000)
        return _make_result(
            check,
            status=CheckStatus.ERROR,
            duration_ms=duration_ms,
            messages=[f"{exc.__class__.__name__}: {exc}"],
            effects_used=tuple(check.effects),
        )
    finally:
        builtins.print = original_print

    errors = list(errors)
    if printed:
        errors.append("check emitted print output; runner policy forbids print()")
    status = CheckStatus.PASS if int(code) == 0 and not errors else CheckStatus.FAIL
    return _make_result(
        check,
        status=status,
        duration_ms=duration_ms,
        messages=errors,
        violations=structured,
        effects_used=tuple(check.effects),
    )


def run_checks(
    ctx: CheckContext | Path,
    selections: CheckSelector | None = None,
    fail_fast: bool = False,
    budget_profile: BudgetProfile | None = None,
    capabilities: Capabilities | None = None,
) -> CheckRunReport:
    if isinstance(ctx, Path):
        from .adapters import FS

        repo_root = ctx.resolve()
        fs = FS(repo_root=repo_root, allowed_roots=(DEFAULT_WRITE_ROOT,))
        context = CheckContext(repo_root=repo_root, fs=fs, env=dict(os.environ))
    else:
        context = ctx
    root_ok, root_errors = ensure_explicit_repo_root(context.repo_root)
    if not root_ok:
        row = CheckResult(
            id="checks_runner_repo_root_policy",
            title="checks runner repo_root policy",
            domain="checks",
            status=CheckStatus.ERROR,
            errors=tuple(root_errors),
            result_code="CHECK_CONTEXT_ROOT_INVALID",
        )
        return CheckRunReport(status="error", rows=(row,), env={"repo_root": str(context.repo_root)})

    selected = list(list_checks())
    if selections is not None:
        selected = select_checks(selected, selections)
    selected = sorted(selected, key=lambda check: str(check.id))

    budget = budget_profile or BudgetProfile()
    effect_policy = (capabilities or Capabilities()).to_effect_policy()
    rows: list[CheckResult] = []
    started = time.perf_counter()
    for check in selected:
        result = _run_single_check(check, context, effect_policy=effect_policy, budget_profile=budget)
        rows.append(result)
        if fail_fast and result.status in {CheckStatus.FAIL, CheckStatus.ERROR}:
            break
    total_duration = int((time.perf_counter() - started) * 1000)
    return CheckRunReport(
        status="ok",
        rows=tuple(rows),
        timings={"duration_ms": total_duration},
        env={"repo_root": str(context.repo_root)},
    )


def top_n_slowest(report: CheckRunReport, n: int = 10) -> list[CheckResult]:
    return sorted(report.rows, key=lambda row: int(row.metrics.get("duration_ms", 0)), reverse=True)[: max(1, int(n))]


def extract_failures(report: CheckRunReport) -> list[CheckResult]:
    return [row for row in report.rows if row.status in {CheckStatus.FAIL, CheckStatus.ERROR}]


def group_failures_by_domain_area(report: CheckRunReport) -> dict[str, dict[str, int]]:
    grouped: dict[str, dict[str, int]] = {}
    for row in extract_failures(report):
        domain = str(row.domain or "unknown")
        rid = str(row.id)
        area = rid.split("_")[2] if rid.startswith("checks_") and len(rid.split("_")) > 2 else "general"
        bucket = grouped.setdefault(domain, {})
        bucket[area] = int(bucket.get(area, 0)) + 1
    return grouped


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
                errors=tuple(str(row.get("detail", row.get("reason", ""))).strip() for _ in [0] if str(row.get("detail", row.get("reason", ""))).strip()),
                metrics={"duration_ms": int(row.get("duration_ms", 0))},
                result_code=str(row.get("result_code", "CHECK_GENERIC")),
            )
        )
    summary = payload.get("summary", {})
    timings = {"duration_ms": int(summary.get("duration_ms", 0))} if isinstance(summary, dict) else {}
    return CheckRunReport(rows=tuple(rows), timings=timings)


__all__ = [
    "BudgetProfile",
    "extract_failures",
    "group_failures_by_domain_area",
    "report_from_payload",
    "run_checks",
    "top_n_slowest",
]

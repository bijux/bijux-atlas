from __future__ import annotations

import builtins
import inspect
import socket
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor, TimeoutError as FutureTimeoutError
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
import os
from typing import Any

from .adapters import FS
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
from .models import EvidenceRef


@dataclass(frozen=True)
class BudgetProfile:
    default_timeout_ms: int = 2_000
    timeout_cap_ms: int = 30_000

    def timeout_for(self, check: CheckDef) -> int:
        budget = int(getattr(check, "budget_ms", 0) or 0)
        effective = budget if budget > 0 else self.default_timeout_ms
        return min(effective, self.timeout_cap_ms)


@dataclass(frozen=True)
class EngineOptions:
    fail_fast: bool = False
    max_failures: int = 0
    only_fast: bool = True
    include_slow: bool = False
    include_internal: bool = False
    quiet: bool = False
    output: str = "text"


def _run_with_timeout(fn: Any, arg: Any, timeout_ms: int) -> Any:
    with ThreadPoolExecutor(max_workers=1) as pool:
        future = pool.submit(fn, arg)
        return future.result(timeout=max(0.001, timeout_ms / 1000.0))


def resolve_run_id(run_id: str | None = None) -> str:
    raw = str(run_id or os.environ.get("ATLASCTL_RUN_ID", "")).strip()
    return raw or "run-deterministic"


def create_check_context(repo_root: Path, *, artifacts_root: Path | None = None, env: dict[str, str] | None = None) -> CheckContext:
    root = repo_root.resolve()
    fs = FS(repo_root=root, allowed_roots=(DEFAULT_WRITE_ROOT,))
    _ = artifacts_root or (root / "artifacts")
    return CheckContext(repo_root=root, fs=fs, env=env or {})


def write_evidence_ref(*, run_id: str, rel_path: str, description: str = "", content_type: str = "text/plain") -> EvidenceRef:
    rid = resolve_run_id(run_id)
    rel = str(rel_path).strip().lstrip("/")
    path = f"artifacts/evidence/{rid}/{rel}"
    return EvidenceRef(kind="artifact", path=path, description=description, content_type=content_type)


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


@contextmanager
def _effect_runtime_guard(check: CheckDef) -> Any:
    allow_subprocess = Effect.SUBPROCESS.value in set(check.effects)
    allow_network = Effect.NETWORK.value in set(check.effects)
    allow_write = Effect.FS_WRITE.value in set(check.effects)

    original_print = builtins.print
    original_open = builtins.open
    original_subprocess_run = subprocess.run
    original_socket_create_connection = socket.create_connection

    printed: list[str] = []

    def _guarded_print(*args: object, **kwargs: object) -> None:  # noqa: ANN401
        printed.append(" ".join(str(item) for item in args))

    def _guarded_open(file: object, mode: str = "r", *args: object, **kwargs: object):  # noqa: ANN401
        if any(token in mode for token in ("w", "a", "x", "+")) and not allow_write:
            raise PermissionError(f"{check.id}: file write usage requires effects to include fs_write")
        return original_open(file, mode, *args, **kwargs)

    def _guarded_subprocess_run(*args: object, **kwargs: object):  # noqa: ANN401
        if not allow_subprocess:
            raise PermissionError(f"{check.id}: subprocess usage requires effects to include subprocess")
        return original_subprocess_run(*args, **kwargs)

    def _guarded_socket_create_connection(*args: object, **kwargs: object):  # noqa: ANN401
        if not allow_network:
            raise PermissionError(f"{check.id}: network usage requires effects to include network")
        return original_socket_create_connection(*args, **kwargs)

    builtins.print = _guarded_print  # type: ignore[assignment]
    builtins.open = _guarded_open  # type: ignore[assignment]
    subprocess.run = _guarded_subprocess_run  # type: ignore[assignment]
    socket.create_connection = _guarded_socket_create_connection  # type: ignore[assignment]
    try:
        yield printed
    finally:
        builtins.print = original_print
        builtins.open = original_open
        subprocess.run = original_subprocess_run
        socket.create_connection = original_socket_create_connection


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

    started = time.perf_counter()
    timeout_ms = budget_profile.timeout_for(check)
    try:
        with _effect_runtime_guard(check) as printed:
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


def _select(
    checks: list[CheckDef],
    selector: CheckSelector | None,
    options: EngineOptions,
) -> list[CheckDef]:
    selected = checks
    if selector is not None:
        selected = select_checks(selected, selector)
    if options.only_fast and not options.include_slow:
        selected = [check for check in selected if not bool(getattr(check, "slow", False))]
    if options.include_slow:
        # no-op: include both fast/slow
        pass
    if not options.include_internal:
        selected = [check for check in selected if "internal" not in set(getattr(check, "tags", ()))]
    return sorted(selected, key=lambda check: str(check.id))


def run_checks(
    registry: Any | None,
    selector: CheckSelector | None,
    context: CheckContext | Path,
    *,
    options: EngineOptions | None = None,
    budget_profile: BudgetProfile | None = None,
    capabilities: Capabilities | None = None,
) -> CheckRunReport:
    if isinstance(context, Path):
        ctx = create_check_context(context)
    else:
        ctx = context

    root_ok, root_errors = ensure_explicit_repo_root(ctx.repo_root)
    if not root_ok:
        row = CheckResult(
            id="checks_runner_repo_root_policy",
            title="checks runner repo_root policy",
            domain="checks",
            status=CheckStatus.ERROR,
            errors=tuple(root_errors),
            result_code="CHECK_CONTEXT_ROOT_INVALID",
        )
        return CheckRunReport(status="error", rows=(row,), env={"repo_root": str(ctx.repo_root)})

    if registry is None:
        base = list(list_checks())
    elif hasattr(registry, "list_checks"):
        base = list(registry.list_checks())
    else:
        base = list(registry)

    opts = options or EngineOptions()
    selected = _select(base, selector, opts)

    budget = budget_profile or BudgetProfile()
    effect_policy = (capabilities or Capabilities()).to_effect_policy()
    rows: list[CheckResult] = []
    failures = 0
    started = time.perf_counter()

    for check in selected:
        if opts.quiet:
            print(f"running {check.id}")
        result = _run_single_check(check, ctx, effect_policy=effect_policy, budget_profile=budget)
        rows.append(result)
        if result.status in {CheckStatus.FAIL, CheckStatus.ERROR}:
            failures += 1
            if opts.fail_fast:
                break
            if opts.max_failures > 0 and failures >= opts.max_failures:
                break

    total_duration = int((time.perf_counter() - started) * 1000)
    return CheckRunReport(
        status="ok",
        rows=tuple(rows),
        timings={"duration_ms": total_duration},
        env={"repo_root": str(ctx.repo_root)},
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


def stable_exit_code(report: CheckRunReport) -> int:
    summary = report.summary
    if int(summary.get("errors", 0)) > 0:
        return 2
    if int(summary.get("failed", 0)) > 0:
        return 1
    return 0


__all__ = [
    "BudgetProfile",
    "create_check_context",
    "EngineOptions",
    "extract_failures",
    "group_failures_by_domain_area",
    "resolve_run_id",
    "run_checks",
    "stable_exit_code",
    "top_n_slowest",
    "write_evidence_ref",
]

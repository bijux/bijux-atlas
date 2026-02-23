from __future__ import annotations

import time
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable

from ..checks.core.base import CheckDef
from ..checks.engine.execution import CommandCheckDef, run_command_checks, run_function_checks
from ..contracts.ids import CHECK_RUN
from ..contracts.validate_self import validate_self

RUNNER_MARKERS = frozenset({"fast", "slow", "network", "write", "lint", "check"})


@dataclass(frozen=True)
class RunnerOptions:
    fail_fast: bool = False
    jobs: int = 1
    timeout_ms: int | None = None
    output: str = "text"  # text|json|quiet|verbose|junit
    dry_run: bool = False
    run_root: Path | None = None
    suite_name: str | None = None
    kind: str = "check-run"
    budget_exceed_behavior: str = "warn"  # warn|fail


@dataclass(frozen=True)
class RunnerEvent:
    seq: int
    event: str
    check_id: str
    status: str
    duration_ms: int = 0
    message: str = ""


def _attach_rel(run_root: Path | None, values: Iterable[str]) -> list[str]:
    out: list[str] = []
    for value in values:
        raw = str(value)
        if not raw:
            continue
        if run_root is None:
            out.append(raw)
            continue
        p = Path(raw)
        try:
            out.append(p.relative_to(run_root).as_posix())
        except Exception:
            out.append(raw)
    return out


def _timing_histogram(rows: list[dict[str, object]]) -> dict[str, int]:
    buckets = {"lt_100ms": 0, "100_500ms": 0, "500_1000ms": 0, "1000_2000ms": 0, "gte_2000ms": 0}
    for row in rows:
        if row.get("status") == "SKIP":
            continue
        d = int(row.get("duration_ms", 0))
        if d < 100:
            buckets["lt_100ms"] += 1
        elif d < 500:
            buckets["100_500ms"] += 1
        elif d < 1000:
            buckets["500_1000ms"] += 1
        elif d < 2000:
            buckets["1000_2000ms"] += 1
        else:
            buckets["gte_2000ms"] += 1
    return buckets


def _marker_set(*, category: str, effects: Iterable[str], duration_ms: int) -> list[str]:
    markers = {category}
    markers.add("slow" if duration_ms >= 1000 else "fast")
    eff = set(str(x) for x in effects)
    if "network" in eff:
        markers.add("network")
    if any(x in eff for x in {"write", "fs-write"}):
        markers.add("write")
    invalid = markers.difference(RUNNER_MARKERS)
    if invalid:
        raise ValueError(f"invalid runner markers: {sorted(invalid)}")
    return sorted(markers)


def _lint_command_policy_violations(cmd: str) -> list[str]:
    low = cmd.lower()
    violations: list[str] = []
    if any(tok in low for tok in (" curl ", " wget ", "http://", "https://")):
        violations.append("lint category forbids network by default")
    if any(tok in cmd for tok in (">", ">>")) or any(tok in low for tok in (" touch ", " mkdir ", " rm ", " cp ", " mv ")):
        violations.append("lint category forbids writes by default")
    return violations


def run_checks_payload(
    repo_root: Path,
    *,
    check_defs: list[CheckDef] | None = None,
    command_defs: list[CommandCheckDef] | None = None,
    run_id: str,
    options: RunnerOptions,
    on_event: Callable[[RunnerEvent], None] | None = None,
) -> tuple[int, dict[str, object]]:
    check_defs = sorted(check_defs or [], key=lambda c: c.check_id)
    command_defs = sorted(list(command_defs or []), key=lambda c: c.check_id)
    all_ids = [c.check_id for c in check_defs] + [c.check_id for c in command_defs]
    if options.dry_run:
        rows = [
            {
                "id": cid,
                "domain": (cid.split("_")[1] if cid.startswith("checks_") and len(cid.split("_")) > 2 else "unknown"),
                "status": "SKIP",
                "duration_ms": 0,
                "reason": "dry-run selection preview",
                "hints": [],
                "owners": [],
                "artifacts": [],
                "category": "check" if cid.startswith("checks_") else "lint",
                "findings": [],
                "attachments": [],
                "markers": ["check"],
                "budget_status": "pass",
            }
            for cid in all_ids
        ]
        payload: dict[str, object] = {
            "schema_name": CHECK_RUN,
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": options.kind,
            "run_id": run_id,
            "status": "ok",
            "summary": {"passed": 0, "failed": 0, "skipped": len(rows), "total": len(rows), "duration_ms": 0},
            "rows": rows,
            "events": [],
            "attachments": [],
            "stream_version": 1,
            "evidence_root": (options.run_root.as_posix() if options.run_root else ""),
            "format_support": ["quiet", "verbose", "json", "text", "junit"],
            "timing_histogram": _timing_histogram(rows),
            "budget_contract": {"exceed_behavior": options.budget_exceed_behavior, "budget_warn_count": 0, "budget_fail_count": 0},
            "marker_contract": {"allowed_markers": sorted(RUNNER_MARKERS)},
        }
        validate_self(CHECK_RUN, payload)
        return 0, payload

    started = time.perf_counter()
    rows: list[dict[str, object]] = []
    events: list[dict[str, object]] = []
    seq = 0
    failed_count = 0

    if check_defs:
        fn_failed, fn_rows = run_function_checks(repo_root, check_defs, timeout_ms=options.timeout_ms, jobs=options.jobs, run_root=options.run_root)
        failed_count += fn_failed
        for row in fn_rows:
            seq += 1
            duration_ms = int(row.metrics.get("duration_ms", 0))
            status = "PASS" if row.status == "pass" else "FAIL"
            reason = "; ".join([*row.errors[:2], *row.warnings[:2]])
            attachments = [{"path": p, "kind": "evidence"} for p in _attach_rel(options.run_root, row.evidence_paths)]
            rows.append(
                {
                    "id": row.id,
                    "title": row.title,
                    "domain": row.domain,
                    "status": status,
                    "duration_ms": duration_ms,
                    "reason": reason,
                    "hints": [row.fix_hint] if row.fix_hint else [],
                    "owners": list(row.owners),
                    "artifacts": [a["path"] for a in attachments],
                    "category": "lint" if "lint" in row.tags else "check",
                    "markers": _marker_set(category=("lint" if "lint" in row.tags else "check"), effects=row.effects, duration_ms=duration_ms),
                    "check_category": row.category,
                    "result_code": row.result_code,
                    "severity": row.severity,
                    "budget_ms": int(row.metrics.get("budget_ms", 0)),
                    "budget_status": str(row.metrics.get("budget_status", "pass")),
                    "findings": [{"code": row.result_code, "message": msg, "hint": row.fix_hint} for msg in row.errors]
                    + [{"code": f"{row.result_code}.WARN", "message": msg, "hint": row.fix_hint} for msg in row.warnings],
                    "attachments": attachments,
                    "writes_allowed_roots": list(row.writes_allowed_roots),
                }
            )
            ev = RunnerEvent(seq=seq, event="check.finish", check_id=row.id, status=status, duration_ms=duration_ms, message=reason)
            events.append(ev.__dict__)
            if on_event:
                on_event(ev)
            if options.fail_fast and status == "FAIL":
                break

    if command_defs and not (options.fail_fast and any(r["status"] == "FAIL" for r in rows)):
        normalized_command_defs: list[CommandCheckDef] = []
        for chk in command_defs:
            cmd = list(chk.cmd)
            if cmd and cmd[0] == "python3":
                cmd[0] = sys.executable
            normalized_command_defs.append(CommandCheckDef(chk.check_id, chk.domain, cmd, chk.budget_ms))
        cmd_failed, cmd_rows = run_command_checks(repo_root, normalized_command_defs)
        # preserve fail-fast semantics after first failing command row
        for row in cmd_rows:
            seq += 1
            status = "PASS" if row["status"] == "pass" else "FAIL"
            if status == "FAIL":
                failed_count += 1
            rid = str(row["id"])
            reason = str(row.get("error", ""))[:500]
            policy_violations = _lint_command_policy_violations(str(row.get("command", "")))
            if policy_violations:
                if reason:
                    reason = "; ".join([reason, *policy_violations])
                else:
                    reason = "; ".join(policy_violations)
                status = "FAIL"
            rows.append(
                {
                    "id": rid,
                    "title": rid,
                    "domain": str(row.get("domain", "lint")),
                    "status": status,
                    "duration_ms": int(row.get("duration_ms", 0)),
                    "reason": reason,
                    "hints": ["Run the lint command directly and fix reported issues."] if status == "FAIL" else [],
                    "owners": [],
                    "artifacts": [],
                    "category": "lint",
                    "markers": _marker_set(category="lint", effects=(), duration_ms=int(row.get("duration_ms", 0))),
                    "check_category": "lint",
                    "result_code": "LINT_COMMAND_FAILED",
                    "severity": "error",
                    "budget_ms": int(row.get("budget_ms", 0)),
                    "budget_status": str(row.get("budget_status", "pass")),
                    "findings": ([{"code": f"{rid}.lint", "message": reason, "hint": "Run underlying lint command."}] if reason else []),
                    "attachments": [],
                    "writes_allowed_roots": ["artifacts/evidence/"],
                    "command": row.get("command", ""),
                }
            )
            ev = RunnerEvent(seq=seq, event="check.finish", check_id=rid, status=status, duration_ms=int(row.get("duration_ms", 0)), message=reason)
            events.append(ev.__dict__)
            if on_event:
                on_event(ev)
            if options.fail_fast and status == "FAIL":
                break
        # if command runner pre-counted more fails than emitted because fail-fast, recompute below
        _ = cmd_failed

    # recompute summary from emitted rows for fail-fast correctness
    failed = sum(1 for r in rows if r["status"] == "FAIL")
    passed = sum(1 for r in rows if r["status"] == "PASS")
    skipped = sum(1 for r in rows if r["status"] == "SKIP")
    duration_ms = int((time.perf_counter() - started) * 1000)
    attachments: list[dict[str, object]] = []
    for r in rows:
        for a in r.get("attachments", []):
            if isinstance(a, dict):
                attachments.append(a)
    budget_warn_count = sum(1 for r in rows if str(r.get("budget_status", "pass")) == "warn")
    budget_fail_count = 0
    if options.budget_exceed_behavior == "fail":
        budget_fail_count = budget_warn_count
        if budget_fail_count:
            failed = max(failed, 1)
    payload = {
        "schema_name": CHECK_RUN,
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": options.kind,
        "run_id": run_id,
        "status": "ok" if failed == 0 else "error",
        "summary": {"passed": passed, "failed": failed, "skipped": skipped, "total": len(rows), "duration_ms": duration_ms},
        "rows": rows,
        "events": events,
        "attachments": attachments,
        "stream_version": 1,
        "evidence_root": (options.run_root.as_posix() if options.run_root else ""),
        "format_support": ["quiet", "verbose", "json", "text", "junit"],
        "timing_histogram": _timing_histogram(rows),
        "budget_contract": {
            "exceed_behavior": options.budget_exceed_behavior,
            "budget_warn_count": budget_warn_count,
            "budget_fail_count": budget_fail_count,
        },
        "marker_contract": {"allowed_markers": sorted(RUNNER_MARKERS)},
    }
    validate_self(CHECK_RUN, payload)
    return (0 if failed == 0 else 1), payload


__all__ = ["RunnerOptions", "RunnerEvent", "run_checks_payload"]

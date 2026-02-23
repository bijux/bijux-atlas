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
    command_defs = list(command_defs or [])
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
        }
        validate_self(CHECK_RUN, payload)
        return 0, payload

    started = time.perf_counter()
    rows: list[dict[str, object]] = []
    events: list[dict[str, object]] = []
    seq = 0
    failed_count = 0

    if check_defs:
        fn_failed, fn_rows = run_function_checks(repo_root, check_defs, timeout_ms=options.timeout_ms, jobs=options.jobs)
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
                    "check_category": row.category,
                    "severity": row.severity,
                    "findings": [{"code": f"{row.id}.error", "message": msg, "hint": row.fix_hint} for msg in row.errors]
                    + [{"code": f"{row.id}.warn", "message": msg, "hint": row.fix_hint} for msg in row.warnings],
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
                    "check_category": "lint",
                    "severity": "error",
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
    }
    validate_self(CHECK_RUN, payload)
    return (0 if failed == 0 else 1), payload


__all__ = ["RunnerOptions", "RunnerEvent", "run_checks_payload"]

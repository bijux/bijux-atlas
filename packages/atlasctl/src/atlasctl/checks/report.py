from __future__ import annotations

import json
from typing import Any

from ..contracts.ids import CHECK_RUN
from ..contracts.validate_self import validate_self
from .model import CheckResult, CheckRunReport


def results_as_rows(results: list[CheckResult]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for result in sorted(results, key=lambda row: row.canonical_key):
        rows.append(
            {
                "id": str(result.id),
                "title": str(result.title),
                "domain": str(result.domain),
                "status": str(result.status).upper(),
                "duration_ms": int(result.metrics.get("duration_ms", 0)),
                "category": str(result.category),
                "result_code": str(result.result_code),
                "owners": list(result.owners),
                "tags": list(result.tags),
                "effects": list(result.effects),
                "violations": [
                    {
                        "code": str(item.code),
                        "message": item.message,
                        "hint": item.hint,
                        "path": item.path,
                        "line": item.line,
                        "column": item.column,
                        "severity": str(item.severity),
                    }
                    for item in result.violations
                ],
                "warnings": list(result.warnings),
                "metrics": dict(result.metrics),
                "evidence_paths": list(result.evidence_paths),
                "hint": str(result.fix_hint),
                "detail": "; ".join(item.message for item in result.violations) or "; ".join(result.errors),
            }
        )
    return rows


def build_report_payload(
    report: CheckRunReport,
    *,
    run_id: str = "",
    tool: str = "atlasctl",
    slow_threshold_ms: int = 800,
    slow_checks: list[dict[str, Any]] | None = None,
    ratchet_errors: list[str] | None = None,
    speed_regressions: list[str] | None = None,
    events: list[dict[str, Any]] | None = None,
    attachments: list[dict[str, Any]] | None = None,
    timing_histogram: dict[str, int] | None = None,
) -> dict[str, Any]:
    rows = results_as_rows(list(report.rows))
    payload: dict[str, Any] = {
        "schema_name": CHECK_RUN,
        "schema_version": 1,
        "tool": tool,
        "kind": "check-run",
        "run_id": run_id,
        "status": str(report.status),
        "summary": dict(report.summary),
        "slow_threshold_ms": int(slow_threshold_ms),
        "slow_checks": list(slow_checks or []),
        "ratchet_errors": list(ratchet_errors or []),
        "speed_regressions": list(speed_regressions or []),
        "rows": rows,
        "events": list(events or []),
        "attachments": list(attachments or []),
        "timing_histogram": timing_histogram or {},
    }
    validate_self(CHECK_RUN, payload)
    return payload


def render_json(payload: dict[str, Any]) -> str:
    return json.dumps(payload, sort_keys=True)


def render_jsonl(payload: dict[str, Any]) -> str:
    lines = [json.dumps({"kind": "check-row", **row}, sort_keys=True) for row in payload.get("rows", [])]
    lines.append(
        json.dumps(
            {
                "kind": "summary",
                "summary": payload.get("summary", {}),
                "slow_threshold_ms": payload.get("slow_threshold_ms", 0),
                "slow_checks": payload.get("slow_checks", []),
                "ratchet_errors": payload.get("ratchet_errors", []),
            },
            sort_keys=True,
        )
    )
    return "\n".join(lines)


def render_text(payload: dict[str, Any], *, quiet: bool = False, verbose: bool = False) -> str:
    rows = sorted(payload.get("rows", []), key=lambda item: str(item.get("id", "")))
    out: list[str] = []
    if quiet:
        for row in rows:
            if row.get("status") == "FAIL":
                out.append(f"FAIL {row['id']}")
        if not out:
            out.append("PASS")
        return "\n".join(out)
    for row in rows:
        if verbose:
            owners = ",".join(row.get("owners", [])) if row.get("owners") else "-"
            line = f"{row.get('status', 'UNKNOWN')} {row.get('id', '')} [{int(row.get('duration_ms', 0))}ms] owners={owners} hint={row.get('hint', '')}"
            out.append(line)
            detail = str(row.get("detail", "")).strip()
            if row.get("status") == "FAIL" and detail:
                out.append(f"  detail: {detail}")
        else:
            out.append(f"{row.get('status', 'UNKNOWN')} {row.get('id', '')} ({int(row.get('duration_ms', 0))}ms)")
    summary = payload.get("summary", {})
    out.append(
        f"summary: passed={int(summary.get('passed', 0))} failed={int(summary.get('failed', 0))} "
        f"skipped={int(summary.get('skipped', 0))} total={int(summary.get('total', 0))} duration_ms={int(summary.get('duration_ms', 0))}"
    )
    if payload.get("slow_checks"):
        out.append(f"slow checks (threshold={int(payload.get('slow_threshold_ms', 0))}ms):")
        for row in payload["slow_checks"][:10]:
            out.append(f"- {row['id']}: {row['duration_ms']}ms")
    for item in payload.get("ratchet_errors", []):
        out.append(f"ratchet: {item}")
    for item in payload.get("speed_regressions", []):
        out.append(f"speed-regression: {item}")
    failed = [row for row in rows if row.get("status") == "FAIL"]
    if failed:
        out.append("failing checks:")
        for row in failed:
            out.append(f"- {row['id']}: {row.get('detail') or row.get('hint')}")
    return "\n".join(out)


def render_show_source(*, check_id: str, source_path: str) -> str:
    payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "id": check_id, "source": source_path}
    return json.dumps(payload, sort_keys=True)


def render_explain(check: Any) -> str:
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "check": {
            "id": str(check.check_id),
            "canonical_id": str(getattr(check, "canonical_id", check.check_id)),
            "domain": str(check.domain),
            "description": str(check.description),
            "intent": str(getattr(check, "intent", "") or check.description),
            "category": str(getattr(check, "category", "")),
            "owners": list(getattr(check, "owners", ()) or ()),
            "tags": list(getattr(check, "tags", ()) or ()),
            "effects": list(getattr(check, "effects", ()) or ()),
            "result_code": str(getattr(check, "result_code", "CHECK_GENERIC")),
            "remediation_short": str(getattr(check, "remediation_short", "")),
            "remediation_link": str(getattr(check, "remediation_link", "")),
        },
    }
    return json.dumps(payload, sort_keys=True)


__all__ = [
    "build_report_payload",
    "render_explain",
    "render_json",
    "render_jsonl",
    "render_show_source",
    "render_text",
    "results_as_rows",
]

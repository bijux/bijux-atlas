from __future__ import annotations

from pathlib import Path

from .execution import run_function_checks
from .registry import list_domains, run_checks_for_domain


def domains() -> list[str]:
    return list_domains()


def run_domain(repo_root: Path, domain: str, fail_fast: bool = False) -> tuple[int, dict[str, object]]:
    selected = run_checks_for_domain(repo_root, domain)
    if not selected:
        return 2, {"schema_version": 1, "status": "fail", "error": f"unknown domain `{domain}`"}

    failed, results = run_function_checks(repo_root, sorted(selected, key=lambda c: c.check_id))
    if fail_fast and failed:
        first_fail = next((i for i, r in enumerate(results) if r.status == "fail"), len(results) - 1)
        results = results[: first_fail + 1]
        failed = 1
    rows: list[dict[str, object]] = []
    for result in results:
        rows.append(
            {
                "id": result.id,
                "domain": result.domain,
                "status": result.status,
                "duration_ms": result.metrics.get("duration_ms", 0),
                "budget_ms": result.metrics.get("budget_ms", 0),
                "budget_status": result.metrics.get("budget_status", "pass"),
                "errors": result.errors,
                "warnings": result.warnings,
                "evidence_paths": result.evidence_paths,
                "metrics": result.metrics,
                "description": result.description,
                "fix_hint": result.fix_hint,
                "category": result.category,
                "severity": result.severity,
            }
        )
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "checks-runner",
        "domain": domain,
        "status": "pass" if failed == 0 else "fail",
        "failed_count": failed,
        "total_count": len(rows),
        "checks": rows,
    }
    return (0 if failed == 0 else 1), payload

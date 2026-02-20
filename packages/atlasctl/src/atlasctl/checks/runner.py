from __future__ import annotations

from pathlib import Path

from .engine import run_function_checks
from .registry import list_domains, run_checks_for_domain


def domains() -> list[str]:
    return list_domains()


def run_domain(repo_root: Path, domain: str) -> tuple[int, dict[str, object]]:
    selected = run_checks_for_domain(repo_root, domain)
    if not selected:
        return 2, {"schema_version": 1, "status": "fail", "error": f"unknown domain `{domain}`"}

    failed, results = run_function_checks(repo_root, selected)
    rows: list[dict[str, object]] = []
    for result in results:
        rows.append(
            {
                "id": result.check_id,
                "domain": result.domain,
                "status": result.status,
                "duration_ms": result.duration_ms,
                "budget_ms": result.budget_ms,
                "budget_status": result.budget_status,
                "errors": result.errors,
                "severity": result.severity,
                "evidence": list(result.evidence),
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

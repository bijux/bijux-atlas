from __future__ import annotations

import time
from pathlib import Path

from .base import CheckDef
from .checks import CHECKS as CHECKS_CHECKS
from .configs import CHECKS as CHECKS_CONFIGS
from .docker import CHECKS as CHECKS_DOCKER
from .docs import CHECKS as CHECKS_DOCS
from .make import CHECKS as CHECKS_MAKE
from .ops import CHECKS as CHECKS_OPS
from .repo import CHECKS as CHECKS_REPO

CHECKS: tuple[CheckDef, ...] = (
    *CHECKS_REPO,
    *CHECKS_MAKE,
    *CHECKS_DOCS,
    *CHECKS_OPS,
    *CHECKS_CHECKS,
    *CHECKS_CONFIGS,
    *CHECKS_DOCKER,
)


def domains() -> list[str]:
    return sorted({"all", *{c.domain for c in CHECKS}})


def run_domain(repo_root: Path, domain: str) -> tuple[int, dict[str, object]]:
    selected = [c for c in CHECKS if domain == "all" or c.domain == domain]
    if not selected:
        return 2, {"schema_version": 1, "status": "fail", "error": f"unknown domain `{domain}`"}

    rows: list[dict[str, object]] = []
    failed = 0
    for chk in selected:
        start = time.perf_counter()
        code, errors = chk.fn(repo_root)
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        status = "pass" if code == 0 else "fail"
        if code != 0:
            failed += 1
        rows.append(
            {
                "id": chk.check_id,
                "domain": chk.domain,
                "status": status,
                "duration_ms": elapsed_ms,
                "budget_ms": chk.budget_ms,
                "budget_status": "pass" if elapsed_ms <= chk.budget_ms else "warn",
                "errors": errors,
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

from __future__ import annotations

import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from ..check.native import (
    check_committed_generated_hygiene,
    check_docs_scripts_references,
    check_duplicate_script_names,
    check_forbidden_top_dirs,
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_help,
    check_make_scripts_references,
    check_no_executable_python_outside_packages,
    check_no_xtask_refs,
    check_ops_generated_tracked,
    check_script_help,
    check_script_ownership,
    check_tracked_timestamp_paths,
)

CheckFunc = Callable[[Path], tuple[int, list[str]]]


@dataclass(frozen=True)
class CheckDef:
    check_id: str
    domain: str
    budget_ms: int
    fn: CheckFunc


CHECKS: tuple[CheckDef, ...] = (
    CheckDef("repo/forbidden-top-dirs", "repo", 500, check_forbidden_top_dirs),
    CheckDef("repo/no-xtask-refs", "repo", 1000, check_no_xtask_refs),
    CheckDef("repo/no-exec-python-outside-packages", "repo", 1500, check_no_executable_python_outside_packages),
    CheckDef("repo/duplicate-script-names", "repo", 1200, check_duplicate_script_names),
    CheckDef("make/scripts-refs", "make", 1000, check_make_scripts_references),
    CheckDef("make/help-determinism", "make", 2000, check_make_help),
    CheckDef("make/forbidden-paths", "make", 1000, check_make_forbidden_paths),
    CheckDef("make/command-allowlist", "make", 1500, check_make_command_allowlist),
    CheckDef("docs/no-scripts-path-refs", "docs", 800, check_docs_scripts_references),
    CheckDef("ops/no-tracked-generated", "ops", 800, check_ops_generated_tracked),
    CheckDef("ops/no-tracked-timestamps", "ops", 1000, check_tracked_timestamp_paths),
    CheckDef("ops/committed-generated-hygiene", "ops", 1000, check_committed_generated_hygiene),
    CheckDef("checks/help-coverage", "checks", 1500, check_script_help),
    CheckDef("checks/ownership-coverage", "checks", 1500, check_script_ownership),
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

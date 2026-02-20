from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from ..core.process import run_command


@dataclass(frozen=True)
class LintCheck:
    check_id: str
    cmd: list[str]


def _py(script: str) -> list[str]:
    return ["python3", script]


def suites(repo_root: Path) -> dict[str, list[LintCheck]]:
    return {
        "ops": [
            LintCheck("ops/no-direct-script-usage", _py("ops/_lint/no-direct-script-usage.py")),
            LintCheck("ops/no-duplicate-readmes", _py("ops/_lint/no-duplicate-readmes.py")),
            LintCheck("ops/no-floating-tool-versions", _py("ops/_lint/no-floating-tool-versions.py")),
            LintCheck("ops/no-orphan-contract", _py("ops/_lint/no-orphan-contract.py")),
            LintCheck("ops/no-orphan-suite", _py("ops/_lint/no-orphan-suite.py")),
        ],
        "repo": [
            LintCheck("repo/no-root-ad-hoc-python", _py("ops/_lint/no-root-ad-hoc-python.py")),
            LintCheck("repo/no-scripts-dir", _py("ops/_lint/no-scripts-dir.py")),
            LintCheck("repo/bin-shims", _py("scripts/areas/check/check-root-bin-shims.py")),
            LintCheck("repo/no-bin-symlinks", ["bash", "ops/_lint/no-bin-symlinks.sh"]),
        ],
        "makefiles": [
            LintCheck("makefiles/safety", _py("scripts/areas/layout/check_make_safety.py")),
            LintCheck("makefiles/public-scripts", _py("scripts/areas/layout/check_make_public_scripts.py")),
        ],
        "docs": [
            LintCheck("docs/no-orphans", _py("scripts/areas/docs/check_no_orphan_docs.py")),
            LintCheck("docs/make-targets", _py("scripts/areas/docs/check_make_targets_documented.py")),
        ],
        "configs": [
            LintCheck("configs/well-formed", _py("scripts/areas/configs/check_config_files_well_formed.py")),
            LintCheck("configs/schemas", _py("scripts/areas/configs/validate_configs_schemas.py")),
        ],
        "packages": [
            LintCheck("packages/atlas-scripts-tests", ["python3", "-m", "pytest", "-q", "packages/bijux-atlas-scripts/tests"]),
        ],
    }


def run_suite(repo_root: Path, suite: str, fail_fast: bool) -> tuple[int, dict[str, object]]:
    all_suites = suites(repo_root)
    checks = all_suites.get(suite)
    if checks is None:
        return 2, {"schema_version": 1, "tool": "bijux-atlas", "suite": suite, "status": "fail", "error": "unknown suite"}

    started_at = datetime.now(timezone.utc).isoformat()
    rows: list[dict[str, object]] = []
    failed = 0
    for check in checks:
        result = run_command(check.cmd, repo_root)
        status = "pass" if result.code == 0 else "fail"
        row: dict[str, object] = {
            "id": check.check_id,
            "command": " ".join(check.cmd),
            "status": status,
        }
        if result.code != 0:
            failed += 1
            row["error"] = result.combined_output
        rows.append(row)
        if result.code != 0 and fail_fast:
            break

    ended_at = datetime.now(timezone.utc).isoformat()
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "suite": suite,
        "status": "pass" if failed == 0 else "fail",
        "started_at": started_at,
        "ended_at": ended_at,
        "checks": rows,
        "failed_count": failed,
        "total_count": len(rows),
    }
    return (0 if failed == 0 else 1), payload

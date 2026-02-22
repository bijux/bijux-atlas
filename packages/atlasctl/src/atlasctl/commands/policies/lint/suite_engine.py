from __future__ import annotations

from pathlib import Path

from ....core.runtime.guards.clock import utc_now_iso
from ....checks.core.execution import CommandCheckDef, run_command_checks


def _py(script: str) -> list[str]:
    return ["python3", script]


def suites(repo_root: Path) -> dict[str, list[CommandCheckDef]]:
    return {
        "ops": [
            CommandCheckDef("ops/no-direct-script-usage", "ops", _py("packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_direct_script_usage.py")),
            CommandCheckDef(
                "ops/no-duplicate-readmes",
                "ops",
                _py("packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_duplicate_readmes.py"),
            ),
            CommandCheckDef("ops/no-floating-tool-versions", "ops", _py("packages/atlasctl/src/atlasctl/commands/ops/lint/policy/no_floating_tool_versions.py")),
            CommandCheckDef("ops/no-orphan-contract", "ops", _py("packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_orphan_contract.py")),
            CommandCheckDef("ops/no-orphan-suite", "ops", _py("packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_orphan_suite.py")),
        ],
        "repo": [
            CommandCheckDef("repo/no-root-ad-hoc-python", "repo", _py("ops/_lint/no-root-ad-hoc-python.py")),
            CommandCheckDef("repo/no-scripts-dir", "repo", _py("ops/_lint/no-scripts-dir.py")),
            CommandCheckDef("repo/bin-shims", "repo", ["python3", "-m", "atlasctl.cli", "check", "root-bin-shims"]),
            CommandCheckDef("repo/no-bin-symlinks", "repo", ["bash", "ops/_lint/no-bin-symlinks.sh"]),
        ],
        "makefiles": [
            CommandCheckDef("makefiles/safety", "makefiles", _py("packages/atlasctl/src/atlasctl/checks/domains/policies/make/check_make_safety.py")),
            CommandCheckDef("makefiles/public-scripts", "makefiles", _py("packages/atlasctl/src/atlasctl/checks/domains/policies/make/check_make_public_scripts.py")),
        ],
        "docs": [
            CommandCheckDef("docs/check", "docs", ["python3", "-m", "atlasctl.cli", "docs", "check", "--report", "json"]),
        ],
        "configs": [
            CommandCheckDef("configs/validate", "configs", ["python3", "-m", "atlasctl.cli", "configs", "validate", "--report", "json"]),
        ],
        "packages": [
            CommandCheckDef("packages/atlas-scripts-tests", "packages", ["python3", "-m", "pytest", "-q", "packages/atlasctl/tests"]),
        ],
    }


def run_lint_suite(repo_root: Path, suite: str, fail_fast: bool) -> tuple[int, dict[str, object]]:
    all_suites = suites(repo_root)
    checks = all_suites.get(suite)
    if checks is None:
        return 2, {"schema_version": 1, "tool": "bijux-atlas", "suite": suite, "status": "fail", "error": "unknown suite"}

    started_at = utc_now_iso()
    failed, rows = run_command_checks(repo_root, checks)
    if fail_fast and failed:
        first_fail = next((i for i, row in enumerate(rows) if row["status"] == "fail"), len(rows) - 1)
        rows = rows[: first_fail + 1]
        failed = 1

    ended_at = utc_now_iso()
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

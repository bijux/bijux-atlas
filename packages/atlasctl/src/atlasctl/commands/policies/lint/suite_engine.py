from __future__ import annotations

from pathlib import Path

from ....core.runtime.guards.clock import utc_now_iso
from ....checks.core.execution import CommandCheckDef
from ....engine.runner import RunnerOptions, run_checks_payload


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
            CommandCheckDef(
                "repo/no-root-ad-hoc-python",
                "repo",
                _py("packages/atlasctl/src/atlasctl/commands/ops/lint/repo/no_root_ad_hoc_python.py"),
            ),
            CommandCheckDef(
                "repo/no-scripts-dir",
                "repo",
                _py("packages/atlasctl/src/atlasctl/commands/ops/lint/repo/no_scripts_dir.py"),
            ),
            CommandCheckDef("repo/bin-shims", "repo", ["python3", "-m", "atlasctl.cli", "check", "root-bin-shims"]),
            CommandCheckDef(
                "repo/no-bin-symlinks",
                "repo",
                _py("packages/atlasctl/src/atlasctl/commands/ops/lint/repo/no_bin_symlinks.py"),
            ),
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
    rc, run_payload = run_checks_payload(
        repo_root,
        command_defs=checks,
        run_id="lint-suite",
        options=RunnerOptions(fail_fast=fail_fast, output="json", kind="check-run"),
    )
    ended_at = utc_now_iso()
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "suite": suite,
        "status": "pass" if rc == 0 else "fail",
        "started_at": started_at,
        "ended_at": ended_at,
        "checks": run_payload.get("rows", []),
        "failed_count": int(run_payload["summary"]["failed"]),  # type: ignore[index]
        "total_count": int(run_payload["summary"]["total"]),  # type: ignore[index]
        "runner": run_payload,
    }
    return rc, payload

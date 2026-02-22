from __future__ import annotations

from pathlib import Path

from atlasctl.checks.registry import get_check


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _run_checks(ids: list[str]) -> None:
    root = _repo_root()
    for check_id in ids:
        check = get_check(check_id)
        assert check is not None, f"missing check: {check_id}"
        code, errors = check.fn(root)
        assert code == 0, [check_id, *errors]


def test_package_shape_and_budget_checks_pass() -> None:
    _run_checks(
        [
            "checks_repo_package_has_module_or_readme",
            "checks_repo_dir_count_trend_gate",
            "checks_repo_budget_drift_approval",
            "checks_repo_module_budget_domains",
        ]
    )


def test_effect_shell_and_reachability_checks_pass() -> None:
    _run_checks(
        [
            "checks_repo_effect_boundaries",
            "checks_repo_effect_boundary_exceptions_policy",
            "checks_repo_shell_strict_mode",
            "checks_repo_shell_no_network_fetch",
            "checks_repo_shell_no_direct_python",
            "checks_repo_shell_invocation_boundary",
            "checks_repo_core_no_bash_subprocess",
            "checks_repo_dead_modules",
            "checks_repo_dead_module_reachability",
        ]
    )

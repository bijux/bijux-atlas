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
            "repo.package_has_module_or_readme",
            "repo.dir_count_trend_gate",
            "repo.budget_drift_approval",
        ]
    )


def test_effect_shell_and_reachability_checks_pass() -> None:
    _run_checks(
        [
            "repo.effect_boundaries",
            "repo.effect_boundary_exceptions_policy",
            "repo.shell_strict_mode",
            "repo.shell_no_network_fetch",
            "repo.shell_no_direct_python",
            "repo.shell_invocation_boundary",
            "repo.dead_modules",
            "repo.dead_module_reachability",
        ]
    )

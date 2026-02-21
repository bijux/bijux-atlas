from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.import_policy import (
    check_command_import_lint,
    check_compileall_gate,
    check_import_smoke,
    check_internal_import_boundaries,
    check_no_modern_imports_from_legacy,
)


def test_import_policy_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    checks = (
        check_internal_import_boundaries,
        check_no_modern_imports_from_legacy,
        check_command_import_lint,
        check_compileall_gate,
        check_import_smoke,
    )
    for check in checks:
        code, errors = check(repo_root)
        assert code == 0, f"{check.__name__}: {errors}"

from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.enforcement.import_policy import (
    check_cli_import_scope,
    check_command_import_lint,
    check_core_no_command_imports,
    check_cold_import_budget,
    check_compileall_gate,
    check_checks_no_cli_imports,
    check_import_smoke,
    check_internal_import_boundaries,
    check_no_modern_imports_from_legacy,
    check_registry_definition_boundary,
)


def test_import_policy_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    checks = (
        check_internal_import_boundaries,
        check_no_modern_imports_from_legacy,
        check_command_import_lint,
        check_core_no_command_imports,
        check_checks_no_cli_imports,
        check_cli_import_scope,
        check_registry_definition_boundary,
        check_compileall_gate,
        check_import_smoke,
        check_cold_import_budget,
    )
    for check in checks:
        code, errors = check(repo_root)
        assert code == 0, f"{check.__name__}: {errors}"


def test_core_no_command_imports_detects_violation(tmp_path: Path) -> None:
    bad = tmp_path / "packages/atlasctl/src/atlasctl/core/bad.py"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text("from atlasctl.commands.check import command\n", encoding="utf-8")
    code, errors = check_core_no_command_imports(tmp_path)
    assert code == 1
    assert any("import-chain core -> atlasctl.commands.check" in err for err in errors)


def test_checks_no_cli_imports_detects_violation(tmp_path: Path) -> None:
    bad = tmp_path / "packages/atlasctl/src/atlasctl/checks/repo/bad.py"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text("from atlasctl.cli.main import main\n", encoding="utf-8")
    code, errors = check_checks_no_cli_imports(tmp_path)
    assert code == 1
    assert any("import-chain checks -> atlasctl.cli.main" in err for err in errors)

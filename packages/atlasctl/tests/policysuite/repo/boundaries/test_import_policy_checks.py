from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.enforcement.import_policy import (
    check_cli_import_scope,
    check_cli_render_boundary,
    check_command_import_lint,
    check_commands_no_check_impl_imports,
    check_core_no_command_imports,
    check_cold_import_budget,
    check_compileall_gate,
    check_checks_no_command_imports,
    check_checks_no_cli_imports,
    check_import_smoke,
    check_internal_import_boundaries,
    check_no_modern_imports_from_legacy,
    check_registry_definition_boundary,
)


def _repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "packages/atlasctl/src/atlasctl").exists():
            return parent
    raise AssertionError("unable to locate repository root")


def test_import_policy_checks_pass_repo() -> None:
    repo_root = _repo_root()
    checks = (
        check_internal_import_boundaries,
        check_no_modern_imports_from_legacy,
        check_command_import_lint,
        check_commands_no_check_impl_imports,
        check_core_no_command_imports,
        check_checks_no_command_imports,
        check_checks_no_cli_imports,
        check_cli_import_scope,
        check_cli_render_boundary,
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


def test_checks_no_command_imports_detects_violation(tmp_path: Path) -> None:
    bad = tmp_path / "packages/atlasctl/src/atlasctl/checks/repo/bad_commands_import.py"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text("from atlasctl.commands.check import command\n", encoding="utf-8")
    code, errors = check_checks_no_command_imports(tmp_path)
    assert code == 1
    assert any("import-chain checks -> atlasctl.commands.check" in err for err in errors)


def test_commands_no_check_impl_imports_detects_violation(tmp_path: Path) -> None:
    bad = tmp_path / "packages/atlasctl/src/atlasctl/commands/dev/bad_check_import.py"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text("from atlasctl.checks.layout.root import check_root_shape\n", encoding="utf-8")
    code, errors = check_commands_no_check_impl_imports(tmp_path)
    assert code == 1
    assert any("import-chain commands -> atlasctl.checks.layout.root" in err for err in errors)


def test_cli_render_boundary_detects_duplicate_renderer(tmp_path: Path) -> None:
    render = tmp_path / "packages/atlasctl/src/atlasctl/cli/render.py"
    render.parent.mkdir(parents=True, exist_ok=True)
    render.write_text("def render_error(*, as_json: bool, message: str, code: int):\n    return message\n", encoding="utf-8")
    dup = tmp_path / "packages/atlasctl/src/atlasctl/cli/output.py"
    dup.write_text("def render_error(*, as_json: bool, message: str, code: int):\n    return message\n", encoding="utf-8")
    code, errors = check_cli_render_boundary(tmp_path)
    assert code == 1
    assert any("render_error must be defined only" in err for err in errors)

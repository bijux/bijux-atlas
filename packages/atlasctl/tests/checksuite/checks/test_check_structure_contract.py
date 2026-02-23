from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.enforcement.structure.check_structure import (
    check_checks_canonical_location,
    check_cli_main_loc_budget,
    check_main_entrypoint_calls_app_main,
)


def test_check_structure_detects_shell_checks(tmp_path: Path) -> None:
    repo = tmp_path
    checks_root = repo / "packages/atlasctl/src/atlasctl/checks"
    checks_root.mkdir(parents=True, exist_ok=True)
    (checks_root / "REGISTRY.toml").write_text('[[checks]]\nid = "checks_repo_ok"\n', encoding="utf-8")
    (checks_root / "legacy.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    code, errors = check_checks_canonical_location(repo)
    assert code == 1
    assert any("migration completeness failed" in msg for msg in errors)


def test_check_structure_detects_duplicate_registry_ids(tmp_path: Path) -> None:
    repo = tmp_path
    checks_root = repo / "packages/atlasctl/src/atlasctl/checks"
    checks_root.mkdir(parents=True, exist_ok=True)
    (checks_root / "check_demo.py").write_text("def demo():\n    return 0\n", encoding="utf-8")
    (checks_root / "REGISTRY.toml").write_text(
        '[[checks]]\nid = "checks_repo_dup"\n\n[[checks]]\nid = "checks_repo_dup"\n',
        encoding="utf-8",
    )
    code, errors = check_checks_canonical_location(repo)
    assert code == 1
    assert any("duplicate check id in registry: checks_repo_dup" in msg for msg in errors)


def test_main_entrypoint_calls_app_main_contract(tmp_path: Path) -> None:
    entry = tmp_path / "packages/atlasctl/src/atlasctl/__main__.py"
    entry.parent.mkdir(parents=True, exist_ok=True)
    entry.write_text(
        "from .app.main import main\n\nif __name__ == \"__main__\":\n    raise SystemExit(main())\n",
        encoding="utf-8",
    )
    code, errors = check_main_entrypoint_calls_app_main(tmp_path)
    assert code == 0
    assert errors == []


def test_cli_main_loc_budget_detects_violation(tmp_path: Path) -> None:
    cli_main = tmp_path / "packages/atlasctl/src/atlasctl/cli/main.py"
    cli_main.parent.mkdir(parents=True, exist_ok=True)
    cli_main.write_text("\n".join("x=1" for _ in range(301)), encoding="utf-8")
    code, errors = check_cli_main_loc_budget(tmp_path)
    assert code == 1
    assert any("LOC budget exceeded" in msg for msg in errors)

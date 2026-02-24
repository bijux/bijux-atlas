from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.enforcement.package_hygiene import check_no_empty_dirs_or_pointless_nests


def test_check_no_empty_dirs_flags_empty_directory(tmp_path: Path) -> None:
    empty_dir = tmp_path / "packages" / "atlasctl" / "src" / "atlasctl" / "docs" / "empty"
    empty_dir.mkdir(parents=True)

    code, errors = check_no_empty_dirs_or_pointless_nests(tmp_path)

    assert code == 1
    assert any("empty directory is forbidden" in error for error in errors)


def test_check_no_empty_dirs_flags_pointless_single_child_nest(tmp_path: Path) -> None:
    nested = tmp_path / "packages" / "atlasctl" / "src" / "atlasctl" / "docs" / "nested" / "leaf"
    nested.mkdir(parents=True)
    (nested / "module.py").write_text("VALUE = 1\n", encoding="utf-8")

    code, errors = check_no_empty_dirs_or_pointless_nests(tmp_path)

    assert code == 1
    assert any("pointless single-child nesting is forbidden" in error for error in errors)

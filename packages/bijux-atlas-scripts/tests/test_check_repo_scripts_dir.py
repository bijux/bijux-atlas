from __future__ import annotations

from pathlib import Path

from bijux_atlas_scripts.checks.repo.scripts_dir import check_scripts_dir_absent


def test_check_scripts_dir_absent_fails_when_scripts_exists(tmp_path: Path) -> None:
    (tmp_path / "scripts").mkdir()
    code, errors = check_scripts_dir_absent(tmp_path)
    assert code == 1
    assert errors == ["forbidden top-level directory exists: scripts/"]


def test_check_scripts_dir_absent_passes_without_scripts(tmp_path: Path) -> None:
    code, errors = check_scripts_dir_absent(tmp_path)
    assert code == 0
    assert errors == []

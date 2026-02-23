from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.enforcement.structure.check_top_level_structure import check_top_level_structure


def _mk_repo(tmp_path: Path) -> Path:
    root = tmp_path / "packages/atlasctl/src/atlasctl"
    for name in ("app", "capability", "cli", "checks", "commands", "contracts", "engine", "registry", "runtime"):
        (root / name).mkdir(parents=True, exist_ok=True)
    return tmp_path


def test_top_level_structure_accepts_expected_shape(tmp_path: Path) -> None:
    repo = _mk_repo(tmp_path)
    code, errors = check_top_level_structure(repo)
    assert code == 0
    assert errors == []


def test_top_level_structure_fails_depth_budget(tmp_path: Path) -> None:
    repo = _mk_repo(tmp_path)
    deep = repo / "packages/atlasctl/src/atlasctl/commands/a/b/c/d/e/f"
    deep.mkdir(parents=True, exist_ok=True)
    code, errors = check_top_level_structure(repo)
    assert code == 1
    assert any("depth" in err for err in errors)


def test_top_level_structure_fails_python_file_budget(tmp_path: Path) -> None:
    repo = _mk_repo(tmp_path)
    target = repo / "packages/atlasctl/src/atlasctl/runtime"
    for idx in range(41):
        (target / f"mod_{idx}.py").write_text("x=1\n", encoding="utf-8")
    code, errors = check_top_level_structure(repo)
    assert code == 1
    assert any("python file budget exceeded" in err for err in errors)

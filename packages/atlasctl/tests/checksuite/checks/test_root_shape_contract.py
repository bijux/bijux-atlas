from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.root_shape import run

_WHITELIST_SRC = Path("packages/atlasctl/src/atlasctl/checks/tools/root_policy.json")


def _touch(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("", encoding="utf-8")


def _mkdir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def _seed_required(root: Path) -> None:
    target_whitelist = root / "packages/atlasctl/src/atlasctl/checks/tools/root_policy.json"
    target_whitelist.parent.mkdir(parents=True, exist_ok=True)
    target_whitelist.write_text(_WHITELIST_SRC.read_text(encoding="utf-8"), encoding="utf-8")
    _mkdir(root / ".cargo")
    _mkdir(root / ".github")
    _touch(root / ".gitignore")
    _touch(root / "Cargo.toml")
    _touch(root / "Cargo.lock")
    _touch(root / "Makefile")
    _touch(root / "README.md")
    _mkdir(root / "crates")
    _mkdir(root / "packages")
    _mkdir(root / "makefiles")
    _mkdir(root / "docs")
    _mkdir(root / "ops")
    _mkdir(root / "configs")


def test_root_shape_passes_with_required_and_allowed_entries(tmp_path: Path) -> None:
    _seed_required(tmp_path)
    _mkdir(tmp_path / "docker")
    _touch(tmp_path / ".dockerignore")
    _touch(tmp_path / "mkdocs.yml")
    _touch(tmp_path / "rust-toolchain.toml")
    _touch(tmp_path / "python-toolchain.toml")
    _touch(tmp_path / "LICENSE")
    _touch(tmp_path / "CHANGELOG.md")
    _touch(tmp_path / "pyproject.toml")
    _mkdir(tmp_path / "artifacts")
    _mkdir(tmp_path / ".venv")
    _mkdir(tmp_path / "dist")

    code, errors = run(tmp_path)
    assert code == 0
    assert errors == []


def test_root_shape_emits_typed_violations(tmp_path: Path) -> None:
    _seed_required(tmp_path)
    (tmp_path / "crates").rmdir()
    _mkdir(tmp_path / "surprise-dir")

    code, errors = run(tmp_path)
    assert code == 1
    assert any(line.startswith("ROOT_SHAPE_MISSING_REQUIRED|entry=crates|") for line in errors)
    assert any(line.startswith("ROOT_SHAPE_UNEXPECTED_ENTRY|entry=surprise-dir|") for line in errors)

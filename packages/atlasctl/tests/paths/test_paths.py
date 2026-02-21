from __future__ import annotations

from pathlib import Path

from atlasctl.paths import find_root, resolve


def test_find_root_resolves_repo() -> None:
    root = find_root(Path(__file__))
    assert (root / "makefiles").is_dir()
    assert (root / "configs").is_dir()


def test_resolve_relative_path_from_repo_root() -> None:
    path = resolve("configs/repo/root-files-allowlist.txt", Path(__file__))
    assert path.exists()
    assert path.name == "root-files-allowlist.txt"

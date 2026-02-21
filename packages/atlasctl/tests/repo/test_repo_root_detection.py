from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from atlasctl.core.repo_root import find_repo_root


def test_find_repo_root_from_nested_file(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    (repo / ".git").mkdir(parents=True)
    (repo / "makefiles").mkdir()
    (repo / "configs").mkdir()
    nested = repo / "packages" / "atlasctl" / "src" / "atlasctl" / "cli" / "main.py"
    nested.parent.mkdir(parents=True)
    nested.write_text("# stub\n", encoding="utf-8")

    assert find_repo_root(nested) == repo


def test_find_repo_root_raises_when_missing_markers(tmp_path: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="atlasctl-no-repo-") as td:
        with pytest.raises(RuntimeError):
            find_repo_root(Path(td))

"""Git adapter helpers."""

from __future__ import annotations

from pathlib import Path

from ..core.git import GitContext, read_git_context


__all__ = ["GitContext", "read_git_context", "get_git_context"]


def get_git_context(repo_root: Path) -> GitContext:
    return read_git_context(repo_root)

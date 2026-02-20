from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .process import run_command


@dataclass(frozen=True)
class GitContext:
    sha: str
    is_dirty: bool


def read_git_context(repo_root: Path) -> GitContext:
    sha_res = run_command(["git", "rev-parse", "--short", "HEAD"], repo_root)
    sha = sha_res.stdout.strip() if sha_res.code == 0 else "unknown"
    dirty_res = run_command(["git", "status", "--porcelain"], repo_root)
    is_dirty = bool(dirty_res.stdout.strip()) if dirty_res.code == 0 else True
    return GitContext(sha=sha or "unknown", is_dirty=is_dirty)

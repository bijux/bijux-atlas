from __future__ import annotations

from pathlib import Path

from ..repo.native import check_ops_generated_tracked


def check_ops_generated_not_tracked_unless_allowed(repo_root: Path) -> tuple[int, list[str]]:
    return check_ops_generated_tracked(repo_root)

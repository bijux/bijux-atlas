from __future__ import annotations

from pathlib import Path


def check_scripts_dir_absent(repo_root: Path) -> tuple[int, list[str]]:
    scripts_dir = repo_root / "scripts"
    if scripts_dir.exists():
        return 1, ["forbidden top-level directory exists: scripts/"]
    return 0, []

from __future__ import annotations

from pathlib import Path

CHECK_ID = "repo.forbidden_root_files"
DESCRIPTION = "forbid junk files at repository root"

_FORBIDDEN_FILES = (".DS_Store", "Thumbs.db")


def run(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for name in _FORBIDDEN_FILES:
        if (repo_root / name).is_file():
            errors.append(f"forbidden root file exists: {name}")
    return (0 if not errors else 1), errors

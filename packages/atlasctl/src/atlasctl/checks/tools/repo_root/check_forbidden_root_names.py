from __future__ import annotations

from pathlib import Path

CHECK_ID = "repo.forbidden_root_names"
DESCRIPTION = "forbid legacy top-level root names"

_FORBIDDEN_NAMES = ("charts", "e2e", "load", "observability", "datasets", "fixtures")


def run(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for name in _FORBIDDEN_NAMES:
        path = repo_root / name
        if path.exists() or path.is_symlink():
            errors.append(f"forbidden root entry exists: {name}")
    return (0 if not errors else 1), errors

from __future__ import annotations

from pathlib import Path

from ....core.repo_root import find_repo_root


def find_root(start: Path | None = None) -> Path:
    return find_repo_root(start)


def resolve(relative_or_abs: str | Path, start: Path | None = None) -> Path:
    raw = Path(relative_or_abs)
    if raw.is_absolute():
        return raw.resolve()
    return (find_repo_root(start) / raw).resolve()

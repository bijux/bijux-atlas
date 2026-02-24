from __future__ import annotations

from pathlib import Path


def iter_files(root: Path, pattern: str = "*") -> list[Path]:
    """Return deterministically sorted file paths under root."""
    if not root.exists():
        return []
    return sorted(path for path in root.rglob(pattern) if path.is_file())


def to_rel_paths(repo_root: Path, paths: list[Path]) -> list[str]:
    """Normalize paths to repo-relative posix form."""
    return sorted(path.resolve().relative_to(repo_root.resolve()).as_posix() for path in paths)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")

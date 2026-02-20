from __future__ import annotations

from pathlib import Path
from typing import Iterable

EXCLUDED_PARTS = {
    ".venv",
    "__pycache__",
    ".mypy_cache",
    ".ruff_cache",
    ".hypothesis",
    "artifacts",
    "target",
}


def iter_files(root: Path, suffixes: Iterable[str]) -> list[Path]:
    wanted = set(suffixes)
    out: list[Path] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        if any(part in EXCLUDED_PARTS for part in path.parts):
            continue
        if path.suffix in wanted:
            out.append(path)
    return out


def iter_python_files(root: Path) -> list[Path]:
    return iter_files(root, {".py"})

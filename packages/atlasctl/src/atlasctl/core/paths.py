from __future__ import annotations

from pathlib import Path


def find_repo_root(start: Path | None = None) -> Path:
    cur = (start or Path(__file__).resolve()).resolve()
    if cur.is_file():
        cur = cur.parent
    while True:
        if (cur / '.git').exists() and (cur / 'makefiles').is_dir() and (cur / 'configs').is_dir():
            return cur
        if cur.parent == cur:
            raise RuntimeError('unable to resolve repository root')
        cur = cur.parent

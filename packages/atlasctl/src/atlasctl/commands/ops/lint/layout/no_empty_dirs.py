#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def _visible_children(path: Path) -> list[Path]:
    return [child for child in path.iterdir() if child.name != ".DS_Store"]


def main() -> int:
    bad = False
    for dir_path in sorted((ROOT / "ops").rglob("*")):
        if not dir_path.is_dir():
            continue
        rel = dir_path.relative_to(ROOT).as_posix()
        if rel.startswith("ops/_artifacts") or rel.startswith("ops/_generated"):
            continue
        children = _visible_children(dir_path)
        if not children:
            print(f"empty directory: {rel}", file=sys.stderr)
            bad = True
            continue
        non_index = [c for c in children if c.name != "INDEX.md"]
        if not non_index and not (dir_path / "INDEX.md").is_file():
            print(f"directory must include INDEX.md explanation: {rel}", file=sys.stderr)
            bad = True
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())

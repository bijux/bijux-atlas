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


def main() -> int:
    bin_dir = ROOT / "bin"
    if not bin_dir.is_dir():
        return 0
    bad = False
    for path in sorted(bin_dir.iterdir()):
        if path.is_symlink():
            print(f"forbidden symlink in root bin/: {path.relative_to(ROOT).as_posix()}", file=sys.stderr)
            bad = True
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())

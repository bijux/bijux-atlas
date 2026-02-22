#!/usr/bin/env python3
from __future__ import annotations

import os
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
STRICT = os.environ.get("STRICT_SCRIPTS_BIN_REMOVAL", "0") == "1"


def main() -> int:
    path = ROOT / "scripts" / "bin"
    if not path.is_dir():
        return 0
    msg = "forbidden legacy directory exists: scripts/bin"
    if STRICT:
        print(msg, file=sys.stderr)
        return 1
    print("warning: scripts/bin still exists (set STRICT_SCRIPTS_BIN_REMOVAL=1 to enforce removal)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

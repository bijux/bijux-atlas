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
STRICT = os.environ.get("STRICT_SCRIPTS_SUNSET", "0") == "1"


def main() -> int:
    scripts_dir = ROOT / "scripts"
    if scripts_dir.exists():
        message = (
            "scripts/ directory exists; scripting SSOT is packages/atlasctl"
        )
        if STRICT:
            print(message, file=sys.stderr)
            return 1
        print(f"warning: {message} (set STRICT_SCRIPTS_SUNSET=1 to enforce)")
    else:
        print("scripts directory check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

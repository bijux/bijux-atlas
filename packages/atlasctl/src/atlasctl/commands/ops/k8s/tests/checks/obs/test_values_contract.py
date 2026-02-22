#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    rc = subprocess.run(["./bin/bijux-atlas", "contracts", "check", "--checks", "chart-values"], cwd=root).returncode
    if rc == 0:
        print("values contract gate passed")
    return rc


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    cmd = ["python3", str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/failure_report.py"), *sys.argv[1:]]
    return subprocess.run(cmd, cwd=str(root), check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())

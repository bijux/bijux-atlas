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
    proc = subprocess.run(
        [
            "rg", "-n",
            "--glob", "*.sh",
            "--glob", "*.mk",
            "--glob", "!packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/obs/test_helm_repo_pinning.py",
            "helm repo add",
            "ops", "makefiles", "scripts",
        ],
        cwd=root,
        text=True,
        capture_output=True,
    )
    if proc.returncode == 0 and proc.stdout.strip():
        print("unpinned helm repo usage detected; use local chart path only", file=sys.stderr)
        print(proc.stdout, file=sys.stderr, end="")
        return 1
    print("helm repo pinning gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

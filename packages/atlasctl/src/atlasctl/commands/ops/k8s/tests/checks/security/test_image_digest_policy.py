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
    cmds = [
        ["python3", str(root / "scripts/areas/check/check-no-latest-tags.py")],
        ["python3", str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/security/test_image_digest_pinning.py")],
    ]
    for cmd in cmds:
        proc = subprocess.run(cmd, cwd=str(root), check=False)
        if proc.returncode != 0:
            return proc.returncode
    print("image digest policy contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

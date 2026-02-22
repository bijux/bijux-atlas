#!/usr/bin/env python3
from __future__ import annotations

import os
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
    cluster = os.environ.get("ATLAS_E2E_CLUSTER_NAME", "bijux-atlas-e2e")
    image = os.environ.get("ATLAS_E2E_LOCAL_IMAGE", "bijux-atlas:local")
    node = f"{cluster}-control-plane"
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN validate image resolution for {image} in {node}")
        return 0
    p1 = subprocess.run(["kind", "load", "docker-image", image, "--name", cluster], cwd=str(root), check=False, text=True, capture_output=True)
    if p1.returncode != 0:
        sys.stderr.write(p1.stderr or p1.stdout)
        return p1.returncode
    p2 = subprocess.run(["docker", "exec", node, "crictl", "images"], cwd=str(root), check=False, text=True, capture_output=True)
    if p2.returncode != 0:
        sys.stderr.write(p2.stderr or p2.stdout)
        return p2.returncode
    if image.split(":")[0] in (p2.stdout or ""):
        print(f"kind image resolution passed: {image}")
        return 0
    print(f"image not present in kind node runtime: {image}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

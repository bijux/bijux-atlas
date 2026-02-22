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
    profile = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("ATLAS_PERF_BASELINE_PROFILE", "local")
    results_dir = os.environ.get("ATLAS_PERF_RESULTS_DIR", "artifacts/perf/results")
    cmd = [
        str(root / "bin/atlasctl"),
        "run",
        "./packages/atlasctl/src/atlasctl/commands/ops/load/baseline/update_baseline.py",
        "--profile",
        str(profile),
        "--results",
        str(results_dir),
        "--environment",
        os.environ.get("ATLAS_PERF_ENVIRONMENT", "local"),
        "--k8s-profile",
        str(profile or "kind"),
        "--replicas",
        os.environ.get("ATLAS_PERF_REPLICAS", "1"),
    ]
    return subprocess.run(cmd, check=False, cwd=str(root)).returncode


if __name__ == "__main__":
    raise SystemExit(main())

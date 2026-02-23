#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
import argparse
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    ap = argparse.ArgumentParser()
    ap.add_argument("profile", nargs="?", default=os.environ.get("ATLAS_PERF_BASELINE_PROFILE", "local"))
    ap.add_argument("--i-know-what-im-doing", action="store_true", dest="ack")
    ap.add_argument("--justification", default="")
    args = ap.parse_args()
    if not args.ack:
        print("refusing baseline update without --i-know-what-im-doing", file=sys.stderr)
        return 2
    if not str(args.justification).strip():
        print("refusing baseline update without --justification", file=sys.stderr)
        return 2
    profile = args.profile
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
        "--justification",
        str(args.justification).strip(),
    ]
    return subprocess.run(cmd, check=False, cwd=str(root)).returncode


if __name__ == "__main__":
    raise SystemExit(main())

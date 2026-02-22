#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys


def main() -> int:
    proc = subprocess.run(
        ["git", "diff", "--name-only", "--cached"],
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        print(proc.stderr.strip() or "git diff --cached failed", file=sys.stderr)
        return 1
    staged = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    touches_baselines = any(path.startswith("configs/ops/perf/baselines/") for path in staged)
    if touches_baselines:
        if os.environ.get("PERF_BASELINE_UPDATE_FLOW", "0") != "1":
            print(
                "baseline update must go through make perf/baseline-update (PERF_BASELINE_UPDATE_FLOW=1 missing)",
                file=sys.stderr,
            )
            return 1
        if os.environ.get("ATLAS_BASELINE_APPROVED", "0") != "1":
            print("baseline update requires explicit approval: set ATLAS_BASELINE_APPROVED=1", file=sys.stderr)
            return 1
    print("baseline policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

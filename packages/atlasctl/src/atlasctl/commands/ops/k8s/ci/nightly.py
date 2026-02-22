from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def main() -> int:
    root = _repo_root()
    steps = [
        ["make", "ops-up"],
        ["make", "ops-reset"],
        ["make", "ops-publish-medium"],
        ["make", "ops-deploy"],
        ["make", "ops-k8s-tests"],
        ["make", "ops-drill-pod-churn"],
        ["make", "ops-report"],
    ]
    for step in steps:
        subprocess.run(step, check=True, cwd=root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

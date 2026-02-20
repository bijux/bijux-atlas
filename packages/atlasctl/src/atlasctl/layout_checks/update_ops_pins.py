#!/usr/bin/env python3
# Purpose: manual pins refresh helper with explicit changelog output.
# Inputs: configs/ops/pins/*.json.
# Outputs: updates configs/ops/pins.json and prints changed lines.
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
    subprocess.check_call(["python3", "packages/atlasctl/src/atlasctl/layout_checks/generate_ops_pins.py"], cwd=ROOT)
    diff = subprocess.check_output(
        ["git", "diff", "--", "configs/ops/pins.json", "configs/ops/pins/"],
        cwd=ROOT,
        text=True,
    )
    if not diff.strip():
        print("pins/update: no changes")
        return 0
    print("pins/update changelog:")
    for line in diff.splitlines():
        if line.startswith(("+", "-")) and not line.startswith(("+++", "---")):
            print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PATHS = [
    "makefiles/targets.json",
    "docs/_generated/make-targets.md",
]


def main() -> int:
    subprocess.run(["python3", "scripts/areas/docs/generate_make_targets_catalog.py"], cwd=ROOT, check=True)
    diff = subprocess.run(["git", "diff", "--", *PATHS], cwd=ROOT, capture_output=True, text=True, check=False)
    if diff.returncode != 0:
        print("make targets catalog drift detected", file=sys.stderr)
        print("- run: python3 scripts/areas/docs/generate_make_targets_catalog.py", file=sys.stderr)
        print(diff.stdout, file=sys.stderr)
        return 1
    print("make targets catalog drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

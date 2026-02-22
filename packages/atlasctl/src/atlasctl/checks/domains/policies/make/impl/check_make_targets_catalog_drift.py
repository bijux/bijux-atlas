#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
PATHS = [
    "makefiles/targets.json",
    "docs/_generated/make-targets.md",
    "artifacts/generated/make/targets.catalog.json",
]


def main() -> int:
    subprocess.run(["python3", "-m", "atlasctl.cli", "docs", "generate-make-targets-catalog", "--report", "text"], cwd=ROOT, check=True)
    diff = subprocess.run(["git", "diff", "--", *PATHS], cwd=ROOT, capture_output=True, text=True, check=False)
    if diff.returncode != 0:
        print("make targets catalog drift detected", file=sys.stderr)
        print("- run: python3 -m atlasctl.cli docs generate-make-targets-catalog --report text", file=sys.stderr)
        print(diff.stdout, file=sys.stderr)
        return 1
    print("make targets catalog drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

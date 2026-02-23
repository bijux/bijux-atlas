#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

from ......core.process import run_command

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
PATHS = [
    "configs/make/targets.json",
    "docs/_generated/make-targets.md",
    "artifacts/generated/make/targets.catalog.json",
]


def main() -> int:
    gen = run_command(["./bin/atlasctl", "docs", "generate-make-targets-catalog", "--report", "text"], cwd=ROOT)
    if gen.code != 0:
        print("make targets catalog generation failed", file=sys.stderr)
        if gen.stderr:
            print(gen.stderr, file=sys.stderr)
        return 1
    diff = run_command(["git", "diff", "--", *PATHS], cwd=ROOT)
    if diff.code != 0:
        print("make targets catalog drift detected", file=sys.stderr)
        print("- run: ./bin/atlasctl docs generate-make-targets-catalog --report text", file=sys.stderr)
        print(diff.stdout, file=sys.stderr)
        return 1
    print("make targets catalog drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

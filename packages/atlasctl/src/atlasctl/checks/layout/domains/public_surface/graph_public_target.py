#!/usr/bin/env python3
from __future__ import annotations

import argparse
from make_target_graph import parse_make_targets, render_tree
import sys
from pathlib import Path

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

from atlasctl.checks.tools.make_public_targets import public_names

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("target")
    args = p.parse_args()

    target = args.target
    if target not in set(public_names()):
        print(f"not public: {target}")
        return 1

    graph = parse_make_targets(ROOT / "makefiles")
    for line in render_tree(graph, target):
        print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

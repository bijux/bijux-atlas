#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from pathlib import Path

from make_target_graph import parse_make_targets, render_tree
from public_make_targets import entry_map

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
LEGACY_TARGET_RE = re.compile(r"(^|/)legacy($|-)")


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("target")
    args = p.parse_args()

    target = args.target
    if LEGACY_TARGET_RE.search(target):
        print(f"legacy target names are forbidden: {target}")
        return 2

    entries = entry_map()
    if target not in entries:
        print(f"not public: {target}")
        return 1

    entry = entries[target]
    print(f"target: {target}")
    print(f"description: {entry['description']}")
    print(f"lanes: {', '.join(entry['lanes'])}")

    graph = parse_make_targets(ROOT / "makefiles")
    print("internal expansion tree:")
    for line in render_tree(graph, target):
        print(f"  {line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

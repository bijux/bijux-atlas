#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

from .public_targets import public_names
from .target_graph import parse_make_targets, render_tree

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: atlasctl-dev-make-graph <target>")
        return 2
    target = sys.argv[1]
    if target not in set(public_names()):
        print(f"not public: {target}")
        return 1

    graph = parse_make_targets(ROOT / "makefiles")
    for line in render_tree(graph, target):
        print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

from .public_targets import public_names
from .target_graph import parse_make_targets, render_tree

ROOT = Path(__file__).resolve().parents[5]


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

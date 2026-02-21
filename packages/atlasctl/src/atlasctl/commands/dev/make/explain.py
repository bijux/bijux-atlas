#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

from .public_targets import entry_map
from .target_graph import parse_make_targets, render_tree

ROOT = Path(__file__).resolve().parents[7]
LEGACY_TARGET_RE = re.compile(r"(^|/)legacy($|-)")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: atlasctl-dev-make-explain <target>")
        return 2
    target = sys.argv[1]
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

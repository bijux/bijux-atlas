#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
TOOLS_SRC = ROOT / "tools" / "bijux-atlas-scripts" / "src"
if str(TOOLS_SRC) not in sys.path:
    sys.path.insert(0, str(TOOLS_SRC))


def main() -> int:
    from bijux_atlas_scripts.layout.boundary_check import check_boundaries

    violations = check_boundaries(ROOT)
    if violations:
        print("bijux-atlas-scripts boundary check failed", file=sys.stderr)
        for v in violations:
            print(f"- {v.file}:{v.line} disallowed import {v.source} -> {v.target}", file=sys.stderr)
        return 1
    print("bijux-atlas-scripts boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

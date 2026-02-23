#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]

if str(ROOT / "packages" / "atlasctl" / "src") not in sys.path:
    sys.path.insert(0, str(ROOT / "packages" / "atlasctl" / "src"))

from atlasctl.commands.ops.runtime_modules.index_generator import render_indexes  # noqa: E402


def main() -> int:
    expected = render_indexes(ROOT)
    drift: list[str] = []
    for rel, text in sorted(expected.items()):
        path = ROOT / rel
        current = path.read_text(encoding="utf-8") if path.exists() else ""
        if current != text:
            drift.append(rel)
    if drift:
        print("ops INDEX generated drift detected:", file=sys.stderr)
        for rel in drift:
            print(f"- {rel}", file=sys.stderr)
        print("run: ./bin/atlasctl ops gen index", file=sys.stderr)
        return 1
    print("ops INDEX generated check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    docs_path = ROOT / "packages/atlasctl/docs/commands/groups/ops.md"
    text = docs_path.read_text(encoding="utf-8")
    bad = []
    if re.search(r"`?ops\\s+internal\\b", text):
        bad.append("docs/commands/groups/ops.md exposes internal ops command in public docs")
    internal_dir = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/internal"
    if internal_dir.exists():
        for p in internal_dir.rglob("*.py"):
            rel = p.relative_to(ROOT).as_posix()
            if rel in text:
                bad.append(f"public ops docs reference internal module path: {rel}")
    if bad:
        print("ops internal/public boundary failed:", file=sys.stderr)
        for line in bad:
            print(line, file=sys.stderr)
        return 1
    print("ops internal/public boundary passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

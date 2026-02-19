#!/usr/bin/env python3
# Purpose: list non-public make targets for maintainers.
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"
TARGET_RE = re.compile(r"^([a-zA-Z0-9_.-]+):(?:\s|$)", flags=re.M)


def main() -> int:
    public = set(json.loads(SURFACE.read_text(encoding="utf-8")).get("make_targets", []))
    all_targets: set[str] = set()
    for mk in sorted((ROOT / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        all_targets.update(t for t in TARGET_RE.findall(text) if not t.startswith("."))
    internal = sorted(t for t in all_targets if t not in public and t != "help")
    for t in internal:
        print(t)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

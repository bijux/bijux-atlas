#!/usr/bin/env python3
# Purpose: list internal make targets for maintainers.
from __future__ import annotations

import re
from pathlib import Path

from public_make_targets import public_names

ROOT = Path(__file__).resolve().parents[6]
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", flags=re.M)


def main() -> int:
    public = set(public_names())
    all_targets: set[str] = set()
    for mk in sorted((ROOT / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        all_targets.update(t for t in TARGET_RE.findall(text) if not t.startswith("."))

    internal = sorted(
        t for t in all_targets if t not in public and (t.startswith("internal/") or t.startswith("_"))
    )
    for target in internal:
        print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

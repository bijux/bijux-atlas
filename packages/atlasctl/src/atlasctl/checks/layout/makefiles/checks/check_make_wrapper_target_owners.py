#!/usr/bin/env python3
# Purpose: enforce ownership metadata for wrapper make targets.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OWNERS = ROOT / "makefiles" / "ownership.json"
WRAPPERS = [
    ROOT / "makefiles" / "dev.mk",
]
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def _targets(path: Path) -> list[str]:
    return [t for t in TARGET_RE.findall(path.read_text(encoding="utf-8")) if not t.startswith(".") and not t.startswith("_") and not t.startswith("internal/")]


def main() -> int:
    ownership = json.loads(OWNERS.read_text(encoding="utf-8"))
    errors: list[str] = []
    for path in WRAPPERS:
        for target in _targets(path):
            meta = ownership.get(target)
            if not isinstance(meta, dict):
                errors.append(f"{path.relative_to(ROOT)}: ownership missing for `{target}`")
                continue
            if not str(meta.get("owner", "")).strip():
                errors.append(f"{path.relative_to(ROOT)}: owner missing for `{target}`")
            if not str(meta.get("area", "")).strip():
                errors.append(f"{path.relative_to(ROOT)}: area missing for `{target}`")

    if errors:
        print("make wrapper ownership check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("make wrapper ownership check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

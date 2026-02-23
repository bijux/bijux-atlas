#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
SURFACE = ROOT / "ops" / "_meta" / "surface.json"
ID_RE = re.compile(r"^ops\.[a-z0-9-]+(?:\.[a-z0-9-]+){1,}$")


def main() -> int:
    try:
        payload = json.loads(SURFACE.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"failed reading {SURFACE.relative_to(ROOT)}: {exc}", file=sys.stderr)
        return 1
    rows = payload.get("actions", [])
    if not isinstance(rows, list):
        print("ops/inventory/surfaces.json: actions must be list", file=sys.stderr)
        return 1
    errors: list[str] = []
    seen: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            errors.append("actions row must be object")
            continue
        aid = str(row.get("id", "")).strip()
        if not ID_RE.fullmatch(aid):
            errors.append(f"invalid action id `{aid}`")
        if aid in seen:
            errors.append(f"duplicate action id `{aid}`")
        seen.add(aid)
    if errors:
        print("ops action taxonomy check failed:", file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("ops action taxonomy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

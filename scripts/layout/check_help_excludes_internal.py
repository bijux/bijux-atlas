#!/usr/bin/env python3
# Purpose: ensure internal targets never appear in curated help/public surface output.
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"


def main() -> int:
    data = json.loads(SURFACE.read_text(encoding="utf-8"))
    public = set(data.get("make_targets", []))
    internal = set()

    # Internal convention: _internal.* or internal/*
    for t in public:
        if t.startswith("_internal.") or t.startswith("internal/"):
            internal.add(t)

    if internal:
        print("help/internal visibility check failed", file=sys.stderr)
        for t in sorted(internal):
            print(f"- internal target leaked into public surface: {t}", file=sys.stderr)
        return 1

    print("help/internal visibility check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

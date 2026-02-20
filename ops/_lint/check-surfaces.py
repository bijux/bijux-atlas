#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CFG = ROOT / "configs/repo/surfaces.json"


def main() -> int:
    cfg = json.loads(CFG.read_text(encoding="utf-8"))
    allowed_dirs = set(cfg["allowed_root_dirs"])
    allowed_files = set(cfg["allowed_root_files"])
    canonical = set(cfg["canonical_surfaces"])

    root_entries = [p.name for p in ROOT.iterdir() if p.name not in {"..", "."}]
    unknown: list[str] = []
    for name in sorted(root_entries):
        path = ROOT / name
        if path.is_dir() and name not in allowed_dirs and name not in canonical:
            unknown.append(name)
        if path.is_file() and name not in allowed_files:
            unknown.append(name)

    missing = sorted(name for name in canonical if not (ROOT / name).exists())

    if unknown or missing:
        print("repo surface check failed:", file=sys.stderr)
        for name in unknown:
            print(f"- unknown root entry: {name}", file=sys.stderr)
        for name in missing:
            print(f"- missing canonical surface: {name}", file=sys.stderr)
        return 1

    print("repo surface check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

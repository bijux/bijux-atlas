#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def main() -> int:
    s = load_surface()
    print("Public Make Surface:")
    for t in s["make_targets"]:
        print(f"  {t}")
    print("Public Ops Run Commands:")
    for c in s["ops_run_commands"]:
        print(f"  {c}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

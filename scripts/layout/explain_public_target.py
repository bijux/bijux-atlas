#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("target")
    args = p.parse_args()
    s = load_surface()
    t = args.target
    if t in s["make_targets"]:
        print(f"public target: {t}")
        return 0
    print(f"not public: {t}")
    print("See: configs/ops/public-surface.json")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

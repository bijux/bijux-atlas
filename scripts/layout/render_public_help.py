#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def render_help(surface: dict) -> None:
    print("Public Make Surface:")
    for t in surface["make_targets"]:
        print(f"  {t}")
    print("Public Ops Run Commands:")
    for c in surface["ops_run_commands"]:
        print(f"  {c}")


def render_gates(surface: dict) -> None:
    core = surface.get("core_targets", [])
    print("Core Public Gates:")
    for t in core:
        print(f"  {t}")


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--mode", choices=["help", "gates", "list-public"], default="help")
    args = p.parse_args()
    s = load_surface()
    if args.mode == "gates":
        render_gates(s)
    elif args.mode == "list-public":
        for t in s["make_targets"]:
            print(t)
    else:
        render_help(s)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

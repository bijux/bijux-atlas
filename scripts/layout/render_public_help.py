#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections import defaultdict

from public_make_targets import public_entries


def render_help(entries: list[dict]) -> None:
    print("Public Make Targets:")
    for entry in entries:
        print(f"  {entry['name']:<14} {entry['description']}")


def render_list(entries: list[dict]) -> None:
    for entry in entries:
        print(f"{entry['name']}: {entry['description']}")


def render_gates(entries: list[dict]) -> None:
    grouped: dict[str, list[str]] = defaultdict(list)
    for entry in entries:
        grouped[entry["area"]].append(entry["name"])
    print("Public Gates by Area:")
    for area in sorted(grouped):
        print(f"  [{area}]")
        for target in grouped[area]:
            print(f"    {target}")


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--mode", choices=["help", "gates", "list"], default="help")
    args = p.parse_args()

    entries = public_entries()
    if args.mode == "gates":
        render_gates(entries)
    elif args.mode == "list":
        render_list(entries)
    else:
        render_help(entries)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

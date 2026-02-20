#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections import defaultdict
from pathlib import Path

from public_make_targets import public_entries
from make_target_graph import parse_make_targets

ROOT = Path(__file__).resolve().parents[2]


def namespace_of(name: str) -> str:
    if "/" in name:
        return name.split("/", 1)[0]
    return "global"


def render_help(entries: list[dict]) -> None:
    grouped: dict[str, list[dict]] = defaultdict(list)
    for entry in entries:
        grouped[namespace_of(entry["name"])].append(entry)
    print("Public Make Targets:")
    for namespace in sorted(grouped):
        print(f"  [{namespace}]")
        for entry in sorted(grouped[namespace], key=lambda item: item["name"]):
            print(f"    {entry['name']:<14} {entry['description']}")


def render_list(entries: list[dict]) -> None:
    for entry in sorted(entries, key=lambda item: (namespace_of(item["name"]), item["name"])):
        print(f"{entry['name']}: {entry['description']}")


def render_gates(entries: list[dict]) -> None:
    grouped: dict[str, list[str]] = defaultdict(list)
    for entry in entries:
        grouped[entry["area"]].append(entry["name"])
    print("Public Gates by Area:")
    for area in sorted(grouped):
        print(f"  [{area}]")
        for target in sorted(grouped[area]):
            print(f"    {target}")

def render_advanced(entries: list[dict]) -> None:
    render_help(entries)
    print("")
    print("Advanced Maintainer Targets:")
    advanced = ["what", "explain", "graph", "list", "report", "report/print", "legacy-audit"]
    for t in advanced:
        print(f"  {t}")


def render_all() -> None:
    graph = parse_make_targets(ROOT / "makefiles")
    for target in sorted(graph):
        print(target)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--mode", choices=["help", "gates", "list", "advanced", "all"], default="help")
    args = p.parse_args()

    entries = public_entries()
    if args.mode == "gates":
        render_gates(entries)
    elif args.mode == "list":
        render_list(entries)
    elif args.mode == "advanced":
        render_advanced(entries)
    elif args.mode == "all":
        render_all()
    else:
        render_help(entries)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from collections import defaultdict
TOOLS_ROOT = Path(__file__).resolve().parents[3] / "makefiles" / "tools"
if str(TOOLS_ROOT) not in sys.path:
    sys.path.insert(0, str(TOOLS_ROOT))

from make_target_graph import parse_make_targets
import sys
from pathlib import Path

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

from atlasctl.checks.tools.make_public_targets import public_entries

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
LEGACY_TARGET_RE = re.compile(r"(^|/)legacy($|-)")


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
    print("Primary Gates:")
    for entry in sorted(entries, key=lambda item: item["name"]):
        lanes = ",".join(entry.get("lanes", []))
        print(f"  {entry['name']:<12} includes={lanes}  {entry['description']}")

def render_advanced(entries: list[dict]) -> None:
    render_help(entries)
    print("")
    print("Advanced Maintainer Targets:")
    advanced = ["what", "explain", "graph", "list", "report", "report/print"]
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

    graph = parse_make_targets(ROOT / "makefiles")
    offenders = [target for target in sorted(graph) if LEGACY_TARGET_RE.search(target)]
    if offenders:
        print("legacy target names are forbidden in makefiles:", flush=True)
        for target in offenders:
            print(f"- {target}", flush=True)
        return 1

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

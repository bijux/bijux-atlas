#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections import defaultdict

GROUPS = {
    "api": "integration",
    "observe": "operations",
    "ops": "operations",
    "load": "operations",
    "audit": "operations",
    "invariants": "integrity",
    "drift": "integrity",
    "reproduce": "integrity",
    "governance": "integrity",
    "system": "integrity",
    "security": "governance",
    "docs": "governance",
    "configs": "governance",
    "check": "verification",
    "checks": "verification",
    "contract": "verification",
    "registry": "verification",
    "suites": "verification",
    "tests": "verification",
}


def parse_help_commands(output: str) -> list[str]:
    commands: list[str] = []
    in_commands = False
    for line in output.splitlines():
        if line.strip() == "Commands:":
            in_commands = True
            continue
        if not in_commands:
            continue
        if not line.strip():
            continue
        if re.match(r"^\s{2}[a-zA-Z0-9_-]+\s", line):
            name = line.strip().split()[0]
            if name not in {"help"}:
                commands.append(name)
        elif line.startswith("Options:"):
            break
    return commands


def run_help(binary: str) -> str:
    proc = subprocess.run([binary, "--help"], capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "failed to run help")
    return proc.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description="Discover command groups from CLI help")
    parser.add_argument("--binary", default="target/debug/bijux-dev-atlas")
    parser.add_argument("--format", choices=["text", "json"], default="text")
    args = parser.parse_args()

    output = run_help(args.binary)
    commands = parse_help_commands(output)

    grouped: dict[str, list[str]] = defaultdict(list)
    for command in commands:
        grouped[GROUPS.get(command, "misc")].append(command)

    for key in grouped:
        grouped[key] = sorted(grouped[key])

    if args.format == "json":
        print(json.dumps(grouped, indent=2, sort_keys=True))
    else:
        for group in sorted(grouped):
            print(f"[{group}]")
            for command in grouped[group]:
                print(f"  {command}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

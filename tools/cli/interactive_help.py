#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess


PROMPTS = [
    ("1", "Show command groups", ["python3", "tools/cli/discover_subcommands.py", "--format", "text"]),
    ("2", "Show command groups as JSON", ["python3", "tools/cli/discover_subcommands.py", "--format", "json"]),
    ("3", "Show full --help", ["target/debug/bijux-dev-atlas", "--help"]),
]


def run(cmd: list[str]) -> None:
    proc = subprocess.run(cmd, text=True, check=False)
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)


def main() -> int:
    parser = argparse.ArgumentParser(description="Interactive help mode for bijux-dev-atlas")
    parser.parse_args()

    print("Select help mode:")
    for key, label, _ in PROMPTS:
        print(f"  {key}. {label}")
    choice = input("Enter selection: ").strip()
    for key, _, cmd in PROMPTS:
        if choice == key:
            run(cmd)
            return 0
    print("invalid selection")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())

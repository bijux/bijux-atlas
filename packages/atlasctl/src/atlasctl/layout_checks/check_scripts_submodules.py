#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--threshold", type=int, default=25, help="max top-level files before submodule dirs are required")
    args = parser.parse_args()

    errors: list[str] = []
    for scripts_dir in sorted((ROOT / "ops").glob("**/scripts")):
        top_level_files = [p for p in scripts_dir.iterdir() if p.is_file()]
        if len(top_level_files) <= args.threshold:
            continue
        subdirs = {p.name for p in scripts_dir.iterdir() if p.is_dir()}
        if not ({"py", "sh", "lib"} & subdirs):
            rel = scripts_dir.relative_to(ROOT).as_posix()
            errors.append(
                f"{rel} has {len(top_level_files)} top-level files (> {args.threshold}) but no py/, sh/, or lib/ submodule directory"
            )

    if errors:
        print("scripts-submodule check failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("scripts-submodule check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

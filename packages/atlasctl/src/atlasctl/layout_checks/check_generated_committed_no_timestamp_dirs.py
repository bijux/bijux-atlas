#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
TARGET = ROOT / "ops" / "_generated_committed"
TS = re.compile(r".*(\d{8}[-T]\d{6}|\d{4}-\d{2}-\d{2}).*")


def main() -> int:
    if not TARGET.exists():
        print("generated-committed timestamp check passed")
        return 0
    violations: list[str] = []
    for path in TARGET.rglob("*"):
        if not path.is_dir():
            continue
        rel = path.relative_to(ROOT).as_posix()
        if TS.match(path.name):
            violations.append(rel)
    if violations:
        print("generated-committed timestamp policy failed", file=sys.stderr)
        for rel in sorted(violations):
            print(f"- timestamped dir under committed generated: {rel}", file=sys.stderr)
        return 1
    print("generated-committed timestamp check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

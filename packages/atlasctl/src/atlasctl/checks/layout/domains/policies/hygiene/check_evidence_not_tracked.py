#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

from atlasctl.core.exec import run

ROOT = Path(__file__).resolve().parents[6]


def main() -> int:
    proc = run(
        ["git", "status", "--short", "artifacts/evidence"],
        cwd=ROOT,
        text=True,
        check=False,
        capture_output=True,
    )
    allowed_suffixes = {
        "artifacts/evidence/.gitkeep",
        "artifacts/evidence/latest-run-id.txt",
    }
    lines = []
    for raw in proc.stdout.splitlines():
        line = raw.strip()
        if not line:
            continue
        path = line.split()[-1]
        if path in allowed_suffixes or path == "artifacts/evidence/":
            continue
        lines.append(line)
    if lines:
        print("evidence policy check failed", file=sys.stderr)
        for line in lines:
            print(f"- tracked evidence change: {line}", file=sys.stderr)
        return 1
    print("evidence policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

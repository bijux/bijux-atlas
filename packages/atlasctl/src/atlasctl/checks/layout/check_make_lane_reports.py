#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: check_make_lane_reports.py <run_id> <lane> [<lane> ...]", file=sys.stderr)
        return 2

    run_id = sys.argv[1]
    lanes = sys.argv[2:]

    missing: list[str] = []
    for lane in lanes:
        path = ROOT / "ops" / "_evidence" / "make" / lane / run_id / "report.json"
        if not path.exists():
            missing.append(str(path.relative_to(ROOT)))

    if missing:
        print("make lane report contract failed", file=sys.stderr)
        for item in missing:
            print(f"- missing lane report: {item}", file=sys.stderr)
        return 1

    print("make lane report contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

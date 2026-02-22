#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

from public_make_targets import load_ssot


def main() -> int:
    data = load_ssot()
    max_targets = int(data.get("max_public_targets", 14))
    target_goal = int(data.get("target_public_targets", max_targets))
    if max_targets > 18:
        print(f"public target hard cap must be <= 18, got {max_targets}", file=sys.stderr)
        return 1
    count = len(data["public_targets"])
    if count > max_targets:
        print(f"public target budget exceeded: {count} > {max_targets}", file=sys.stderr)
        return 1
    if count > target_goal:
        print(
            f"public target budget check passed (hard={max_targets}) but above goal: {count} > {target_goal}",
            file=sys.stderr,
        )
    print(f"public target budget check passed: {count}/{max_targets} (goal={target_goal})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

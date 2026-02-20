#!/usr/bin/env python3
from __future__ import annotations

import sys

from public_make_targets import load_ssot


def main() -> int:
    data = load_ssot()
    max_targets = int(data.get("max_public_targets", 14))
    target_goal = int(data.get("target_public_targets", max_targets))
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

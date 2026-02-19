#!/usr/bin/env python3
from __future__ import annotations

import sys

from public_make_targets import load_ssot


def main() -> int:
    data = load_ssot()
    max_targets = int(data.get("max_public_targets", 14))
    count = len(data["public_targets"])
    if count > max_targets:
        print(f"public target budget exceeded: {count} > {max_targets}", file=sys.stderr)
        return 1
    print(f"public target budget check passed: {count}/{max_targets}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# Purpose: ensure internal targets never appear in curated help/public surface output.
from __future__ import annotations

import sys

from public_make_targets import public_names


def main() -> int:
    leaked = [t for t in public_names() if t.startswith("internal/") or t.startswith("_")]
    if leaked:
        print("help/internal visibility check failed", file=sys.stderr)
        for target in sorted(leaked):
            print(f"- internal target leaked into public surface: {target}", file=sys.stderr)
        return 1

    print("help/internal visibility check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

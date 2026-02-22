#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

from atlasctl.checks.repo.domains.forbidden_adjectives import check_forbidden_adjectives

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    code, errors = check_forbidden_adjectives(ROOT)
    if errors:
        print("forbidden adjective check failed", file=sys.stderr)
        for err in errors[:100]:
            print(f"- {err}", file=sys.stderr)
        return code
    print("forbidden adjective check passed")
    return code


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

from public_make_targets import public_entries


def normalize(entry: dict) -> tuple:
    return (entry["description"].strip().lower(), tuple(sorted(entry.get("lanes", []))), entry["area"])


def main() -> int:
    entries = public_entries()
    by_sig: dict[tuple, list[str]] = {}
    for entry in entries:
        by_sig.setdefault(normalize(entry), []).append(entry["name"])

    errors: list[str] = []
    for names in by_sig.values():
        if len(names) <= 1:
            continue
        errors.append(f"possible duplicate public aliases: {', '.join(sorted(names))}")

    if errors:
        print("public alias duplication check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("public alias duplication check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

import sys

from public_make_targets import ALLOWED_AREAS, entry_map, load_ownership


def main() -> int:
    entries = entry_map()
    ownership = load_ownership()

    errors: list[str] = []
    for target, entry in entries.items():
        meta = ownership.get(target)
        if not meta:
            errors.append(f"ownership missing for public target: {target}")
            continue
        owner = meta.get("owner")
        area = meta.get("area")
        if not owner:
            errors.append(f"owner missing for public target: {target}")
        if area not in ALLOWED_AREAS:
            errors.append(f"invalid area for {target}: {area} (allowed: {', '.join(sorted(ALLOWED_AREAS))})")
        if area != entry.get("area"):
            errors.append(f"area mismatch for {target}: ssot={entry.get('area')} ownership={area}")

    extra = sorted(set(ownership) - set(entries))
    for target in extra:
        errors.append(f"ownership has unknown target: {target}")

    covered = sum(1 for t in entries if t in ownership and ownership[t].get("owner") and ownership[t].get("area"))
    total = len(entries)
    coverage = (covered / total * 100.0) if total else 100.0
    if covered != total:
        errors.append(f"ownership coverage must be 100%: {covered}/{total} ({coverage:.1f}%)")

    if errors:
        print("make target ownership check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print(f"make target ownership check passed: {covered}/{total} (100.0%)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

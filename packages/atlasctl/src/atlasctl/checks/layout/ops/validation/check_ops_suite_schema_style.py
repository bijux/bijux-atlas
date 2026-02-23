#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
SUITES = (
    "ops/load/suites/suites.json",
    "ops/obs/suites/suites.json",
    "ops/e2e/suites/suites.json",
)


def main() -> int:
    errors: list[str] = []
    for rel in SUITES:
        path = ROOT / rel
        if not path.exists():
            errors.append(f"missing suite file: {rel}")
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            errors.append(f"{rel}: payload must be object")
            continue
        if not isinstance(payload.get("schema_version"), int):
            errors.append(f"{rel}: schema_version must be integer")
        suites = payload.get("suites")
        if not isinstance(suites, list):
            errors.append(f"{rel}: suites must be list")
            continue
        for i, row in enumerate(suites):
            if not isinstance(row, dict):
                errors.append(f"{rel}: suites[{i}] must be object")
                continue
            rid = str(row.get("id") or row.get("name") or "").strip()
            desc = str(row.get("description") or row.get("purpose") or "").strip()
            if not rid:
                errors.append(f"{rel}: suites[{i}] missing id/name")
            if not desc:
                errors.append(f"{rel}: suites[{i}] missing description/purpose")
    if errors:
        print("ops suite schema-style check failed:", file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("ops suite schema-style check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

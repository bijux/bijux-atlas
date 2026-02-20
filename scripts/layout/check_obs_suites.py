#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = ROOT / "ops/_schemas/obs/suites.schema.json"
SUITES = ROOT / "ops/obs/suites/suites.json"
TEST_DIR = ROOT / "ops/obs/tests"
LEGACY = ROOT / "ops/obs/tests/suites.json"


def main() -> int:
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    suites = json.loads(SUITES.read_text(encoding="utf-8"))
    errors: list[str] = []

    for key in schema.get("required", []):
        if key not in suites:
            errors.append(f"missing required key: {key}")

    ids: set[str] = set()
    for i, suite in enumerate(suites.get("suites", [])):
        sid = suite.get("id")
        if not isinstance(sid, str) or re.match(r"^[a-z0-9-]+$", sid) is None:
            errors.append(f"suite[{i}] invalid id")
            continue
        if sid in ids:
            errors.append(f"duplicate suite id: {sid}")
        ids.add(sid)
        tests = suite.get("tests", [])
        if not tests:
            errors.append(f"suite `{sid}` has no tests")
        for t in tests:
            if not (TEST_DIR / t).exists():
                errors.append(f"suite `{sid}` references missing test script: {t}")

    if LEGACY.exists():
        legacy = json.loads(LEGACY.read_text(encoding="utf-8"))
        if legacy != suites:
            errors.append("ops/obs/tests/suites.json must mirror ops/obs/suites/suites.json")

    if errors:
        print("obs suites contract failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    print("obs suites contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

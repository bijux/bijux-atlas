#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SUITES = ROOT / "ops/load/suites/suites.json"


def main() -> int:
    payload = json.loads(SUITES.read_text(encoding="utf-8"))
    by_name = {s.get("name"): s for s in payload.get("suites", []) if isinstance(s, dict)}
    errors: list[str] = []

    abuse = by_name.get("response-size-abuse")
    if not abuse:
        errors.append("missing required suite: response-size-abuse")
    else:
        run_in = set(abuse.get("run_in", []))
        if "nightly" not in run_in and "load-nightly" not in run_in:
            errors.append("response-size-abuse must run in nightly profile")
        if not abuse.get("must_pass", False):
            errors.append("response-size-abuse must have must_pass=true")

    if errors:
        print("abuse scenario contract failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("abuse scenario contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# Purpose: compare trace contract structure with golden snapshot.
# Inputs: docs/contracts/TRACE_SPANS.json + ops/obs/contract/trace-structure.golden.json
# Outputs: non-zero on structure drift.
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "docs" / "contracts" / "TRACE_SPANS.json"
GOLDEN = ROOT / "ops" / "observability" / "contract" / "trace-structure.golden.json"


def main() -> int:
    c = json.loads(CONTRACT.read_text(encoding="utf-8"))
    g = json.loads(GOLDEN.read_text(encoding="utf-8"))
    current = {
        "schema_version": 1,
        "required_root": c.get("request_root_span", {}),
        "required_spans": sorted([s["name"] for s in c.get("spans", [])]),
        "required_taxonomy": sorted(c.get("taxonomy", [])),
        "required_slow_query_fields": sorted(c.get("slow_query_event", {}).get("required_fields", [])),
    }
    if current != g:
        print("trace golden structure drift detected", file=sys.stderr)
        print(f"expected: {GOLDEN}", file=sys.stderr)
        return 1
    print("trace golden check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

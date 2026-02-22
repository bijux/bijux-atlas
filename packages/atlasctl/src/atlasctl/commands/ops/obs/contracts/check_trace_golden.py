#!/usr/bin/env python3
# Purpose: compare trace contract structure with golden snapshot.
# Inputs: docs/contracts/TRACE_SPANS.json + ops/obs/contract/trace-structure.golden.json
# Outputs: non-zero on structure drift.
from __future__ import annotations

import json
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
CONTRACT = ROOT / "docs" / "contracts" / "TRACE_SPANS.json"
GOLDEN = ROOT / "ops" / "obs" / "contract" / "trace-structure.golden.json"


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

#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OPENAPI = ROOT / "configs/openapi/v1/openapi.generated.json"
CONTRACT = ROOT / "ops/obs/contract/endpoint-observability-contract.json"
TRACE_CONTRACT = ROOT / "docs/contracts/TRACE_SPANS.json"


def main() -> int:
    spec = json.loads(OPENAPI.read_text(encoding="utf-8"))
    coverage = json.loads(CONTRACT.read_text(encoding="utf-8"))
    trace = json.loads(TRACE_CONTRACT.read_text(encoding="utf-8"))

    endpoints = {
        (path, method)
        for path, methods in spec.get("paths", {}).items()
        if path.startswith("/v1/")
        for method in methods.keys()
    }
    covered = {(e["path"], e["method"]) for e in coverage.get("endpoints", [])}
    missing = sorted(endpoints - covered)
    errors: list[str] = []
    if missing:
        for p, m in missing:
            errors.append(f"missing endpoint trace coverage entry: {m.upper()} {p}")

    span_names = {trace.get("request_root_span", {}).get("name", "")}
    span_names.update(s.get("name", "") for s in trace.get("spans", []))
    for entry in coverage.get("endpoints", []):
        for span in entry.get("required_trace_spans", []):
            if span not in span_names:
                errors.append(f"unknown trace span `{span}` for endpoint {entry.get('method')} {entry.get('path')}")

    if errors:
        print("endpoint trace coverage check failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    print("endpoint trace coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

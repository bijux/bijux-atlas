#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OPENAPI = ROOT / "configs/openapi/v1/openapi.generated.json"
CONTRACT = ROOT / "ops/obs/contract/endpoint-observability-contract.json"
TRACE_CONTRACT = ROOT / "docs/contracts/TRACE_SPANS.json"
OBS_BUDGETS = ROOT / "configs/ops/obs/budgets.json"


def main() -> int:
    spec = json.loads(OPENAPI.read_text(encoding="utf-8"))
    coverage = json.loads(CONTRACT.read_text(encoding="utf-8"))
    trace = json.loads(TRACE_CONTRACT.read_text(encoding="utf-8"))
    budgets = json.loads(OBS_BUDGETS.read_text(encoding="utf-8"))

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
    root_required_attrs = set(trace.get("request_root_span", {}).get("required_attributes", []))
    span_required_attrs = set()
    for span in trace.get("spans", []):
        span_required_attrs.update(span.get("required_attributes", []))
    known_attrs = root_required_attrs | span_required_attrs
    class_attr_requirements = budgets.get("span_attribute_requirements", {})
    for entry in coverage.get("endpoints", []):
        klass = entry.get("class")
        needed_attrs = set(class_attr_requirements.get(klass, []))
        missing_attrs = sorted(needed_attrs - known_attrs)
        if missing_attrs:
            errors.append(
                f"endpoint {entry.get('method')} {entry.get('path')} class `{klass}` requires unknown trace attrs: "
                + ", ".join(missing_attrs)
            )
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

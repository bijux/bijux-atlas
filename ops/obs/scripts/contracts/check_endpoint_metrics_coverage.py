#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OPENAPI = ROOT / "configs/openapi/v1/openapi.generated.json"
CONTRACT = ROOT / "ops/obs/contract/endpoint-observability-contract.json"
METRICS_CONTRACT = ROOT / "ops/obs/contract/metrics-contract.json"
OBS_BUDGETS = ROOT / "configs/ops/obs/budgets.json"


def main() -> int:
    spec = json.loads(OPENAPI.read_text(encoding="utf-8"))
    coverage = json.loads(CONTRACT.read_text(encoding="utf-8"))
    metrics = json.loads(METRICS_CONTRACT.read_text(encoding="utf-8"))
    budgets = json.loads(OBS_BUDGETS.read_text(encoding="utf-8"))
    known_metrics = set(metrics.get("required_metric_specs", {}).keys())
    required_by_class = budgets.get("endpoint_class_metric_requirements", {})

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
            errors.append(f"missing endpoint coverage entry: {m.upper()} {p}")

    for entry in coverage.get("endpoints", []):
        klass = entry.get("class")
        if klass not in {"cheap", "medium", "heavy"}:
            errors.append(f"invalid endpoint class `{klass}` for {entry.get('method')} {entry.get('path')}")
        class_required = set(required_by_class.get(klass, []))
        endpoint_metrics = set(entry.get("required_metrics", []))
        missing_class_required = sorted(class_required - endpoint_metrics)
        if missing_class_required:
            errors.append(
                f"endpoint {entry.get('method')} {entry.get('path')} missing class-required metrics: "
                + ", ".join(missing_class_required)
            )
        for metric in entry.get("required_metrics", []):
            if metric not in known_metrics:
                errors.append(f"unknown metric `{metric}` for endpoint {entry.get('method')} {entry.get('path')}")

    if errors:
        print("endpoint metric coverage check failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1
    print("endpoint metric coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

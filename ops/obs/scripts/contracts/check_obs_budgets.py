#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
BUDGETS = ROOT / "configs/ops/obs/budgets.json"
METRICS_CONTRACT = ROOT / "ops/obs/contract/metrics-contract.json"


def main() -> int:
    errors: list[str] = []
    budgets = json.loads(BUDGETS.read_text(encoding="utf-8"))
    metrics = json.loads(METRICS_CONTRACT.read_text(encoding="utf-8"))
    metric_specs = metrics.get("required_metric_specs", {})

    for metric, labels in budgets.get("required_metric_labels", {}).items():
        spec = metric_specs.get(metric)
        if not isinstance(spec, dict):
            errors.append(f"obs budget references unknown metric `{metric}`")
            continue
        spec_labels = set(spec.get("required_labels", []))
        missing = sorted(set(labels) - spec_labels)
        if missing:
            errors.append(
                f"metric `{metric}` missing required labels from budget: {', '.join(missing)}"
            )

    if errors:
        print("observability budgets check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("observability budgets check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

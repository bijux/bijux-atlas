#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
metrics_file = ROOT / "artifacts/e2e/k6/metrics.prom"

if not metrics_file.exists():
    print("runtime metrics file missing; run make e2e-perf first (skipping runtime checks)")
    sys.exit(0)

text = metrics_file.read_text()

def metric_value(name: str):
    m = re.search(rf"^{re.escape(name)}\{{[^\n]*\}}\s+([0-9.eE+-]+)$", text, re.MULTILINE)
    if not m:
        return None
    return float(m.group(1))

required = [
    "bijux_overload_shedding_active",
    "bijux_dataset_hits",
    "bijux_store_breaker_open",
]
missing = [m for m in required if metric_value(m) is None]
if missing:
    print("runtime metrics missing:", file=sys.stderr)
    for m in missing:
        print(f"- {m}", file=sys.stderr)
    sys.exit(1)

# Warmup should produce cache hits.
if metric_value("bijux_dataset_hits") <= 0:
    print("expected bijux_dataset_hits > 0 after warmup", file=sys.stderr)
    sys.exit(1)

print("runtime metrics checks passed")
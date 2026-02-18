#!/usr/bin/env python3
# Purpose: compute SLO burn from k6 score artifacts and metrics snapshot.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
k6 = ROOT / "artifacts/ops/e2e/k6/score.md"
metrics = ROOT / "artifacts/ops/observability/metrics.prom"
out = ROOT / "artifacts/ops/observability/slo-burn.json"
out.parent.mkdir(parents=True, exist_ok=True)

violations = 0
if k6.exists() and "Violations" in k6.read_text():
    violations = 1

store_open = 0
if metrics.exists() and "bijux_store_breaker_open" in metrics.read_text():
    store_open = 1

burn_exceeded = bool(violations)
payload = {
    "schema_version": 1,
    "k6_violations": violations,
    "store_circuit_open_seen": store_open,
    "burn_exceeded": burn_exceeded,
}
out.write_text(json.dumps(payload, indent=2) + "\n")
print(out)

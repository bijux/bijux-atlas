#!/usr/bin/env python3
# Purpose: compute SLO burn from k6 score artifacts and metrics snapshot.
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
k6 = ROOT / "artifacts/ops/e2e/k6/score.md"
metrics = ROOT / "artifacts/ops/obs/metrics.prom"
out = ROOT / "artifacts/ops/obs/slo-burn.json"
out.parent.mkdir(parents=True, exist_ok=True)

violations = 0
if k6.exists() and "Violations" in k6.read_text():
    violations = 1

store_open = 0
metrics_text = metrics.read_text() if metrics.exists() else ""
if "bijux_store_breaker_open" in metrics_text:
    store_open = 1

total = 0.0
errors_5xx = 0.0
for line in metrics_text.splitlines():
    if not line.startswith("bijux_http_requests_total{"):
        continue
    try:
        val = float(line.rsplit(" ", 1)[-1])
    except Exception:
        continue
    total += val
    m = re.search(r'status="([0-9]{3})"', line)
    if m and m.group(1).startswith("5"):
        errors_5xx += val

error_rate = (errors_5xx / total) if total > 0 else 0.0
burn_rate = (error_rate / 0.005) if total > 0 else 0.0
burn_exceeded = bool(violations or burn_rate > 1.0)
payload = {
    "schema_version": 1,
    "k6_violations": violations,
    "store_circuit_open_seen": store_open,
    "http_total_requests": total,
    "http_5xx_requests": errors_5xx,
    "error_rate": error_rate,
    "burn_rate": burn_rate,
    "burn_exceeded": burn_exceeded,
}
out.write_text(json.dumps(payload, indent=2) + "\n")
print(out)

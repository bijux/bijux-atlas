#!/usr/bin/env python3
# Purpose: enforce metric cardinality guardrails by banning risky labels.
from __future__ import annotations
import json
import re
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
BUDGETS = ROOT / "configs/ops/obs/budgets.json"
metrics_path = Path(sys.argv[1]) if len(sys.argv)>1 else ROOT / 'artifacts/ops/obs/metrics.prom'
if not metrics_path.exists():
    print('metrics snapshot not found for cardinality check', file=sys.stderr)
    raise SystemExit(1)
text = metrics_path.read_text()
budgets = json.loads(BUDGETS.read_text(encoding="utf-8"))
forbidden = set(budgets.get("cardinality", {}).get("forbidden_labels", []))
max_series = int(budgets.get("cardinality", {}).get("max_series_per_metric", 2000))
series_counts: dict[str, int] = {}
for line in text.splitlines():
    if not line or line.startswith("#"):
        continue
    metric = line.split("{", 1)[0].strip()
    series_counts[metric] = series_counts.get(metric, 0) + 1
    m = re.search(r'\{([^}]*)\}', line)
    if not m:
        continue
    labels = [kv.split('=')[0].strip() for kv in m.group(1).split(',') if '=' in kv]
    bad = sorted(set(labels) & forbidden)
    if bad:
        print(f'cardinality violation labels {bad} in line: {line}', file=sys.stderr)
        raise SystemExit(1)
for metric, count in sorted(series_counts.items()):
    if count > max_series:
        print(
            f"cardinality violation metric `{metric}` has {count} series > budget {max_series}",
            file=sys.stderr,
        )
        raise SystemExit(1)
print('metric cardinality guardrail passed')

#!/usr/bin/env python3
# Purpose: enforce metric cardinality guardrails by banning risky labels.
from __future__ import annotations
import re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
metrics_path = Path(sys.argv[1]) if len(sys.argv)>1 else ROOT / 'artifacts/ops/observability/metrics.prom'
if not metrics_path.exists():
    print('metrics snapshot not found for cardinality check', file=sys.stderr)
    raise SystemExit(1)
text = metrics_path.read_text()
forbidden = {'request_id', 'cursor', 'gene_id', 'tx_id', 'api_key', 'dataset_hash', 'artifact_hash'}
for line in text.splitlines():
    m = re.search(r'\{([^}]*)\}', line)
    if not m:
        continue
    labels = [kv.split('=')[0].strip() for kv in m.group(1).split(',') if '=' in kv]
    bad = sorted(set(labels) & forbidden)
    if bad:
        print(f'cardinality violation labels {bad} in line: {line}', file=sys.stderr)
        raise SystemExit(1)
print('metric cardinality guardrail passed')

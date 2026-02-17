#!/usr/bin/env python3
# Purpose: validate load result artifacts against result contract and metadata policy.
# Inputs: result summary files under an output directory and contract schema.
# Outputs: non-zero exit on contract violations.
from __future__ import annotations
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / 'artifacts/perf/results'
schema = json.loads((ROOT / 'ops/load/contracts/result-schema.json').read_text())
suite_manifest = json.loads((ROOT / 'ops/load/suites/suites.json').read_text())
required_metrics = set(schema['properties']['metrics']['required'])
errors = []

expected_by_result = {}
for s in suite_manifest.get('suites', []):
    if s.get('kind') == 'k6':
        scenario = s.get('scenario', '')
        stem = Path(scenario).stem
        if stem:
            expected_by_result[stem] = set(s.get('expected_metrics', []))
    elif s.get('kind') == 'script':
        expected_by_result[s.get('name', '')] = set(s.get('expected_metrics', []))

for f in sorted(OUT.glob('*.summary.json')):
    data = json.loads(f.read_text())
    metrics = data.get('metrics', {})
    missing = sorted(required_metrics - set(metrics.keys()))
    if missing:
        errors.append(f"{f}: missing metrics keys {missing}")
    expected_metrics = expected_by_result.get(f.stem.replace('.summary', ''), set())
    for key in sorted(expected_metrics):
        if key not in metrics:
            errors.append(f"{f}: missing expected metric '{key}' from suite manifest")
    meta = f.with_suffix('.meta.json')
    if not meta.exists():
        errors.append(f"{f}: missing metadata sidecar {meta.name}")
        continue
    m = json.loads(meta.read_text())
    for k in ('git_sha', 'image_digest', 'dataset_hash', 'dataset_release', 'policy_hash'):
        if k not in m:
            errors.append(f"{meta}: missing field {k}")

if errors:
    print('load result contract validation failed:', file=sys.stderr)
    for e in errors:
        print(f'- {e}', file=sys.stderr)
    raise SystemExit(1)
print('load result contract validation passed')

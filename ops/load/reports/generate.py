#!/usr/bin/env python3
# Purpose: generate markdown summary report from load result artifacts.
# Inputs: results directory with *.summary.json and *.meta.json
# Outputs: markdown report under artifacts/ops/load/reports/
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RES = ROOT / 'artifacts/perf/results'
OUT = ROOT / 'artifacts/ops/load/reports'
OUT.mkdir(parents=True, exist_ok=True)

def bar(ms: float, max_ms: float = 3000.0) -> str:
    blocks = int(max(0.0, min(1.0, ms / max_ms)) * 10)
    return "#" * blocks + "-" * (10 - blocks)


lines = [
    '# Load Summary Report',
    '',
    '| scenario | p95(ms) | p99(ms) | latency-shape | fail_rate | git_sha | image_digest | dataset_hash | dataset_release | policy_hash |',
    '|---|---:|---:|---|---:|---|---|---|---|---|'
]
for f in sorted(RES.glob('*.summary.json')):
    d = json.loads(f.read_text())
    m = d.get('metrics', {})
    dur = m.get('http_req_duration', {}).get('values', {})
    fail = m.get('http_req_failed', {}).get('values', {})
    meta = f.with_suffix('.meta.json')
    meta_d = json.loads(meta.read_text()) if meta.exists() else {}
    lines.append(
        f"| {f.stem.replace('.summary','')} | {float(dur.get('p(95)',0.0)):.2f} | {float(dur.get('p(99)',0.0)):.2f} | {bar(float(dur.get('p(95)',0.0)))} | {float(fail.get('rate',0.0)):.4f} | {meta_d.get('git_sha','unknown')} | {meta_d.get('image_digest','unknown')} | {meta_d.get('dataset_hash','unknown')} | {meta_d.get('dataset_release','unknown')} | {meta_d.get('policy_hash','unknown')} |"
    )

(OUT / 'summary.md').write_text('\n'.join(lines) + '\n')
print(OUT / 'summary.md')

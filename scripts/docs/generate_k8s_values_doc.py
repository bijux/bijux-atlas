#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
src = ROOT / 'docs' / 'contracts' / 'CHART_VALUES.json'
out = ROOT / 'docs' / 'operations' / 'k8s' / 'values.md'

data = json.loads(src.read_text(encoding='utf-8'))
keys = data.get('top_level_keys', [])

lines = [
    '# Kubernetes Values',
    '',
    '- Owner: `bijux-atlas-operations`',
    '',
    '## What',
    '',
    'Generated summary of Helm top-level values from the chart-values contract.',
    '',
    '## Why',
    '',
    'Keeps operations docs aligned to chart values SSOT.',
    '',
    '## Scope',
    '',
    'Top-level chart values keys only.',
    '',
    '## Non-goals',
    '',
    'Does not redefine schema semantics beyond contract references.',
    '',
    '## Contracts',
]
for k in keys:
    lines.append(f'- `values.{k}`')
lines += [
    '',
    '## Failure modes',
    '',
    'Missing or stale keys can break deployments and profile docs.',
    '',
    '## How to verify',
    '',
    '```bash',
    '$ python3 scripts/docs/generate_k8s_values_doc.py',
    '$ python3 scripts/contracts/check_chart_values_contract.py',
    '```',
    '',
    'Expected output: generated values doc and chart contract check pass.',
    '',
    '## See also',
    '',
    '- [Chart Values Contract](../../contracts/chart-values.md)',
    '- [Helm Chart Contract](chart.md)',
    '- [K8s Index](INDEX.md)',
    ''
]
out.write_text('\n'.join(lines), encoding='utf-8')
print(f'generated {out}')

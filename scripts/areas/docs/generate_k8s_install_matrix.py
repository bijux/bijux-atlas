#!/usr/bin/env python3
# Purpose: generate k8s release install matrix doc from CI artifact summary.
# Inputs: artifacts/ops/k8s-install-matrix.json
# Outputs: docs/operations/k8s/release-install-matrix.md
from __future__ import annotations
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
src = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / 'artifacts/ops/k8s-install-matrix.json'
out = ROOT / 'docs/operations/k8s/release-install-matrix.md'
if not src.exists():
    data = {"generated_at": "unknown", "profiles": [], "tests": []}
else:
    data = json.loads(src.read_text())

lines = [
    '# Release Install Matrix',
    '',
    '- Owner: `bijux-atlas-operations`',
    '',
    '## What',
    '',
    'Generated matrix of k8s install/test profiles from CI summaries.',
    '',
    '## Why',
    '',
    'Provides a stable compatibility view across supported chart profiles.',
    '',
    '## Contracts',
    '',
    f"Generated at: `{data.get('generated_at','unknown')}`",
    '',
    'Profiles:',
]
for p in data.get('profiles', []):
    lines.append(f'- `{p}`')
lines += ['', 'Verified test groups:']
for t in data.get('tests', []):
    lines.append(f'- `{t}`')
lines += [
    '',
    '## Failure modes',
    '',
    'Missing profile/test entries indicate CI generation drift or skipped suites.',
    '',
    '## How to verify',
    '',
    '```bash',
    '$ ops/k8s/ci/install-matrix.sh',
    '$ make docs',
    '```',
    '',
    'Expected output: matrix doc updated and docs checks pass.',
    '',
    '## See also',
    '',
    '- [K8s Test Contract](k8s-test-contract.md)',
    '- [Helm Chart Contract](chart.md)',
    '- `ops-k8s-tests`',
]
out.write_text('\n'.join(lines) + '\n')
print(f'wrote {out}')

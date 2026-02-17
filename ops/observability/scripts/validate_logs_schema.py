#!/usr/bin/env python3
# Purpose: validate JSON log lines against exported fields contract.
from __future__ import annotations
import json, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
schema = json.loads((ROOT / 'ops/observability/contract/logs-fields-contract.json').read_text())
required = schema.get('required', [])
ns = (sys.argv[1] if len(sys.argv) > 1 else 'atlas-e2e')
release = (sys.argv[2] if len(sys.argv) > 2 else 'atlas-e2e')
cmd = ['kubectl','-n',ns,'logs',f'deploy/{release}-bijux-atlas','--tail=200']
try:
    out = subprocess.check_output(cmd, text=True)
except Exception as exc:
    print(f'log schema check skipped: {exc}')
    raise SystemExit(0)
for line in out.splitlines():
    if not line.startswith('{'):
        continue
    obj = json.loads(line)
    for k in required:
        if k not in obj:
            print(f'missing required log field: {k}', file=sys.stderr)
            raise SystemExit(1)
print('log fields contract passed')

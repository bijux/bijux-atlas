#!/usr/bin/env python3
# Purpose: build artifacts index.json for UI/inspection use.
# Inputs: artifacts/ tree.
# Outputs: artifacts/index.json summary.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
art = ROOT / 'artifacts'
art.mkdir(exist_ok=True)
ops = art / 'ops'
entries = []
if ops.exists():
    for p in sorted([x for x in ops.iterdir() if x.is_dir() and x.name != 'latest']):
        entries.append({'name': p.name, 'path': str(p.relative_to(ROOT))})
latest = None
latest_link = ops / 'latest'
if latest_link.exists() and latest_link.is_symlink():
    latest = latest_link.resolve().name
payload = {'ops_runs': entries, 'latest': latest}
(art / 'index.json').write_text(json.dumps(payload, indent=2) + '\n')
print('artifacts/index.json updated')

#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: build deterministic dataset manifest lock with checksums.
# stability: internal
# called-by: ops-datasets-lock
from __future__ import annotations
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
manifest = json.loads((ROOT / 'ops/datasets/manifest.json').read_text())
entries = []
for ds in manifest['datasets']:
    if ds.get('paths'):
        checksums = {}
        for key, rel in sorted(ds['paths'].items()):
            p = ROOT / rel
            if not p.exists():
                checksums[key] = None
                continue
            checksums[key] = hashlib.sha256(p.read_bytes()).hexdigest()
        entries.append({'name': ds['name'], 'id': ds['id'], 'checksums': checksums})
    else:
        entries.append({'name': ds['name'], 'id': ds['id'], 'checksums': {}})
out = {'schema_version': 1, 'entries': entries}
(ROOT / 'ops/datasets/manifest.lock').write_text(json.dumps(out, indent=2) + '\n')
print('wrote ops/datasets/manifest.lock')

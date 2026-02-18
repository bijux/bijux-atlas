#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
THRESHOLDS = json.loads((ROOT / 'configs/ops/cache-thresholds.json').read_text())
STATUS_FILE = ROOT / 'artifacts/ops/cache-status.json'

if not STATUS_FILE.exists():
    print(f'missing cache status artifact: {STATUS_FILE}', file=sys.stderr)
    raise SystemExit(1)

status = json.loads(STATUS_FILE.read_text())
min_hit_ratio = float(THRESHOLDS.get('min_hit_ratio', 0.0))
max_disk_bytes = int(THRESHOLDS.get('max_disk_bytes', 0))

hit_ratio = float(status.get('cache_hit_ratio', 0.0))
disk_bytes = int(status.get('cache_disk_bytes', 0))

if hit_ratio < min_hit_ratio:
    raise SystemExit(f'cache hit ratio below threshold: {hit_ratio:.4f} < {min_hit_ratio:.4f}')
if max_disk_bytes > 0 and disk_bytes > max_disk_bytes:
    raise SystemExit(f'cache disk bytes above threshold: {disk_bytes} > {max_disk_bytes}')

print(f'cache threshold check passed: hit_ratio={hit_ratio:.4f}, disk_bytes={disk_bytes}')

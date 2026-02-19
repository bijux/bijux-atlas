#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: enforce local cache disk budget policy.
# stability: public
# called-by: make ops-cache-status
from __future__ import annotations
import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[4]
cfg = json.loads((root / 'configs/ops/cache-budget-thresholds.v1.json').read_text())
budget = int(cfg.get('max_disk_bytes', 0))
try:
    out = subprocess.check_output("du -sk artifacts/e2e-store 2>/dev/null | awk '{print $1*1024}'", shell=True, text=True).strip()
    usage = int(out or '0')
except Exception:
    usage = 0
if budget and usage > budget:
    raise SystemExit(f'cache budget exceeded: {usage} > {budget}')
print(f'cache budget check passed: {usage}/{budget}')

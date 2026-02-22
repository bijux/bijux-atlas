#!/usr/bin/env python3
# Purpose: safely clean artifact outputs with retention policy.
# Inputs: artifacts/ops run directories.
# Outputs: removes old runs, preserves latest and recent N.
from __future__ import annotations

import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
keep = 5
ops = ROOT / 'artifacts' / 'ops'
if not ops.exists():
    print('no artifacts/ops directory')
    raise SystemExit(0)
latest_name = None
latest_link = ops / 'latest'
if latest_link.exists() and latest_link.is_symlink():
    latest_name = latest_link.resolve().name
runs = sorted([p for p in ops.iterdir() if p.is_dir() and p.name != 'latest'])
for p in runs[:-keep]:
    if p.name == latest_name:
        continue
    shutil.rmtree(p, ignore_errors=True)
    print(f'removed {p.relative_to(ROOT)}')
print('artifacts cleanup complete')

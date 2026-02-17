#!/usr/bin/env python3
# Purpose: verify pinned query lock hashes match the query SSOT file.
# Inputs: ops/load/queries/pinned-v1.json and ops/load/queries/pinned-v1.lock
# Outputs: exit non-zero if lock drift is detected.
from __future__ import annotations
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
src = ROOT / 'ops/load/queries/pinned-v1.json'
lock_path = ROOT / 'ops/load/queries/pinned-v1.lock'

queries = json.loads(src.read_text())
lock = json.loads(lock_path.read_text())

file_hash = hashlib.sha256(src.read_bytes()).hexdigest()
if lock.get('file_sha256') != file_hash:
    print('pinned query lock drift: file hash mismatch', file=sys.stderr)
    sys.exit(1)

expected = {}
for group in ('cheap','heavy'):
    for q in queries.get(group, []):
        expected[q] = hashlib.sha256(q.encode()).hexdigest()
if lock.get('query_hashes') != expected:
    print('pinned query lock drift: query hash mismatch', file=sys.stderr)
    sys.exit(1)

print('pinned query lock check passed')

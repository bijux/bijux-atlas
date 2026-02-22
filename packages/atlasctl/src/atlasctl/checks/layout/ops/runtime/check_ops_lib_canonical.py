#!/usr/bin/env python3
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
ALLOWED = {
    'ops/k8s/tests/checks/_lib/common.sh',
    'ops/k8s/tests/checks/_lib/k8s-suite-lib.sh',
    'ops/k8s/tests/checks/_lib/k8s-contract-lib.sh',
    'ops/load/tests/common.sh',
    'ops/obs/tests/common.sh',
    'ops/obs/tests/observability-test-lib.sh',
}
RET = 'packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh'


def main() -> int:
    errors: list[str] = []
    if (ROOT / 'ops/_lib/common.sh').exists():
        errors.append(f'ops/_lib/common.sh (retired; use {RET})')
    for p in sorted((ROOT / 'ops').rglob('common.sh')):
        rel = p.relative_to(ROOT).as_posix()
        if rel not in ALLOWED:
            errors.append(rel)
    if errors:
        print('ops shell helper canonicalization policy failed:', file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print('ops shell helper canonicalization policy passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS = ROOT / 'ops'

REQUIRED_DIRS = [
    'stack','k8s','load','obs','datasets','inventory','schema','env','observe','_meta','_generated.example','_artifacts','e2e'
]
OPTIONAL_DIRS = {'_generated','_evidence','_examples','fixtures','manifests','registry','report','vendor','docs','helm','kind'}
ALLOWED_FILES = {'README.md','CONTRACT.md','INDEX.md','ERRORS.md'}


def main() -> int:
    for d in REQUIRED_DIRS:
        if not (OPS / d).is_dir():
            print(f'missing required ops directory: ops/{d}', file=sys.stderr)
            return 1
    for entry in sorted(OPS.iterdir()):
        name = entry.name
        if entry.is_dir():
            if name in REQUIRED_DIRS or name in OPTIONAL_DIRS:
                continue
            print(f'forbidden: unexpected ops/ root entry: ops/{name}', file=sys.stderr)
            return 1
        if entry.is_file() and name in ALLOWED_FILES:
            continue
        print(f'forbidden: unexpected ops/ root entry: ops/{name}', file=sys.stderr)
        return 1
    if (OPS / 'e2e/stack').is_symlink():
        print('forbidden: ops/e2e/stack symlink is not allowed; use real directories only', file=sys.stderr)
        return 1
    manifests=[p.relative_to(ROOT).as_posix() for p in (OPS/'e2e').rglob('*') if p.is_file() and p.suffix in {'.yaml','.yml'}]
    if manifests:
        print('forbidden: manifest files found under ops/e2e; keep stack manifests under ops/stack', file=sys.stderr)
        for m in manifests:
            print(m, file=sys.stderr)
        return 1
    print('ops workspace layout check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())

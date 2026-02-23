#!/usr/bin/env python3
from __future__ import annotations
import re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[8]
BASE = ROOT/'packages/atlasctl/src/atlasctl/commands/ops'
ALLOW_OPS_WRITES = {
    'packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/index_generator.py',
    'packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_commands.py',
    'packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/ops_runtime_run.py',
}
OPS_WRITE_RE = re.compile(r'ops/(?!_generated/)[^\s"\']+')
GEN_WRITE_RE = re.compile(r'ops/_generated/')
GEN_GATE_HINT_RE = re.compile(r'ops gen|atlasctl ops gen')

def main() -> int:
    errs=[]
    for path in sorted(BASE.rglob('*.py')):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding='utf-8', errors='ignore')
        if GEN_WRITE_RE.search(text) and rel not in ALLOW_OPS_WRITES:
            errs.append(f'{rel}: refs/writes ops/_generated outside approved generation modules')
        if 'write_text(' in text and 'ops/' in text and rel not in ALLOW_OPS_WRITES:
            if OPS_WRITE_RE.search(text):
                errs.append(f'{rel}: writes to ops/ outside ops/_generated generation modules')
    if errs:
        print('\n'.join(sorted(set(errs))), file=sys.stderr); return 1
    print('ops generation write guards passed'); return 0

if __name__ == '__main__':
    raise SystemExit(main())

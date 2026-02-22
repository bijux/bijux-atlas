#!/usr/bin/env python3
from __future__ import annotations
import os, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def _git_ls_files() -> list[str]:
    p = subprocess.run(['git', 'ls-files'], cwd=ROOT, text=True, capture_output=True)
    return [x for x in p.stdout.splitlines() if x]


def main() -> int:
    fail = False
    tracked = _git_ls_files()
    def bad_lines(pred, msg):
        nonlocal fail
        hits = [x for x in tracked if pred(x)]
        if hits:
            print(msg, file=sys.stderr)
            for h in hits:
                print(h, file=sys.stderr)
            fail = True
    bad_lines(lambda x: x.endswith('/.DS_Store') or x == '.DS_Store', 'tracked .DS_Store files are forbidden')
    bad_lines(lambda x: x.startswith('.idea/'), 'tracked .idea files are forbidden')
    bad_lines(lambda x: x.startswith('target/'), 'tracked target/ files are forbidden')
    if os.environ.get('CI') and (ROOT / 'target').is_dir():
        print('root target/ directory must not exist in CI workspaces', file=sys.stderr)
        fail = True
    ds = [p for p in ROOT.rglob('.DS_Store') if '.git/' not in p.as_posix() and not p.as_posix().endswith('/.git/.DS_Store')]
    if ds:
        print('workspace contains .DS_Store; remove it', file=sys.stderr)
        for p in ds:
            print(str(p.relative_to(ROOT)), file=sys.stderr)
        fail = True
    if fail:
        return 1
    print('repo hygiene check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())

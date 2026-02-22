#!/usr/bin/env python3
from __future__ import annotations
import subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]


def main() -> int:
    gitignore = (ROOT / '.gitignore').read_text(encoding='utf-8', errors='ignore') if (ROOT / '.gitignore').exists() else ''
    if '/artifacts/' not in gitignore.splitlines():
        print('artifacts policy failed: .gitignore must include /artifacts/', file=sys.stderr)
        return 1
    proc = subprocess.run(['git', 'ls-files', 'artifacts'], cwd=ROOT, text=True, capture_output=True)
    tracked = [x for x in proc.stdout.splitlines() if x.strip()]
    bad = [x for x in tracked if x != 'artifacts/.gitkeep']
    if bad:
        print('artifacts policy failed: tracked artifacts payloads are forbidden', file=sys.stderr)
        for x in bad:
            print(x, file=sys.stderr)
        return 1
    print('artifacts policy check passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())

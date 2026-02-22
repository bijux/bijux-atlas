#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('namespace', nargs='?', default='atlas-e2e')
    ap.add_argument('out', nargs='?', default='artifacts/ops/stack/events.txt')
    args = ap.parse_args()
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        ['kubectl', 'get', 'events', '-n', args.namespace, '--sort-by=.lastTimestamp'],
        text=True,
        capture_output=True,
        check=False,
    )
    out.write_text(proc.stdout if proc.returncode == 0 else '', encoding='utf-8')
    print(out)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

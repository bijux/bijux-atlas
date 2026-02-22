#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def _pod_names(ns: str) -> list[str]:
    proc = subprocess.run(
        ['kubectl', '-n', ns, 'get', 'pods', '-o', 'json'],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return []
    try:
        payload = json.loads(proc.stdout)
    except Exception:
        return []
    out: list[str] = []
    for item in payload.get('items', []):
        meta = item.get('metadata', {}) if isinstance(item, dict) else {}
        name = meta.get('name')
        if isinstance(name, str) and name:
            out.append(name)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('namespace', nargs='?', default='atlas-e2e')
    ap.add_argument('out_dir', nargs='?', default='artifacts/ops/stack/logs')
    args = ap.parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    for pod in _pod_names(args.namespace):
        proc = subprocess.run(
            ['kubectl', '-n', args.namespace, 'logs', pod, '--tail=2000'],
            text=True,
            capture_output=True,
            check=False,
        )
        (out_dir / f'{pod}.log').write_text(proc.stdout if proc.returncode == 0 else '', encoding='utf-8')
    print(out_dir)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

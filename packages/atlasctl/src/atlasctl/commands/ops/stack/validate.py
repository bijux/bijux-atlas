#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _can_local_kubectl_validate() -> bool:
    try:
        p1 = subprocess.run(['kubectl', 'version', '--client'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if p1.returncode != 0:
            return False
        p2 = subprocess.run(['kubectl', 'cluster-info'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return p2.returncode == 0
    except FileNotFoundError:
        return False


def main() -> int:
    root = _repo_root()
    local_validate = _can_local_kubectl_validate()
    for f in sorted((root / 'ops/stack').rglob('*.yaml')):
        path = str(f)
        text = f.read_text(encoding='utf-8', errors='ignore')
        if '\t' in text:
            print(f'tab character found in YAML: {f}', file=sys.stderr)
            return 1
        if '/stack/values/' in path or '/stack/kind/cluster' in path:
            continue
        if local_validate:
            p = subprocess.run(['kubectl', 'apply', '--dry-run=client', '--validate=false', '-f', str(f)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if p.returncode != 0:
                print(f'kubectl local validate failed: {f}', file=sys.stderr)
                return 1
        try:
            kc = subprocess.run(['kubeconform', '-strict', '-summary', str(f)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if kc.returncode != 0:
                print(f'kubeconform validate failed: {f}', file=sys.stderr)
                return 1
        except FileNotFoundError:
            pass
    print('stack manifest validation passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

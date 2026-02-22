#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _stack_ns_default(root: Path) -> str:
    contract = json.loads((root / 'ops/_meta/layer-contract.json').read_text(encoding='utf-8'))
    return str(contract.get('namespaces', {}).get('stack', 'atlas-e2e'))


def _svc_name(root: Path, key: str) -> str:
    contract = json.loads((root / 'ops/_meta/layer-contract.json').read_text(encoding='utf-8'))
    return str(contract.get('services', {}).get(key, {}).get('service_name', key))


def _run(cmd: list[str]) -> None:
    p = subprocess.run(cmd, text=True, capture_output=True, check=False)
    if p.returncode != 0:
        if p.stdout:
            print(p.stdout, end='')
        if p.stderr:
            print(p.stderr, end='', file=sys.stderr)
        raise SystemExit(p.returncode)


def main() -> int:
    root = _repo_root()
    ap = argparse.ArgumentParser()
    ap.add_argument('namespace', nargs='?', default=_stack_ns_default(root))
    ap.add_argument('timeout', nargs='?', default='180s')
    args = ap.parse_args()
    ns = args.namespace
    timeout = args.timeout
    _run(['kubectl', 'wait', '--for=condition=Ready', 'nodes', '--all', f'--timeout={timeout}'])
    _run(['kubectl', '-n', 'kube-system', 'rollout', 'status', 'deploy/coredns', f'--timeout={timeout}'])
    for svc_key in ('minio', 'prometheus'):
        name = _svc_name(root, svc_key)
        _run(['kubectl', '-n', ns, 'wait', '--for=condition=available', f'deploy/{name}', f'--timeout={timeout}'])
    print(f'stack ready: ns={ns}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

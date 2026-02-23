#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _contract(root: Path) -> dict:
    return json.loads((root / 'ops/inventory/layers.json').read_text(encoding='utf-8'))


def _svc(root: Path, key: str) -> str:
    return str(_contract(root).get('services', {}).get(key, {}).get('service_name', key))


def _run(cmd: list[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    p = subprocess.run(cmd, text=True, capture_output=True, check=False)
    if check and p.returncode != 0:
        if p.stdout:
            print(p.stdout, end='')
        if p.stderr:
            print(p.stderr, end='', file=sys.stderr)
        raise SystemExit(p.returncode)
    return p


def main() -> int:
    root = _repo_root()
    contract = _contract(root)
    ns_default = str(contract.get('namespaces', {}).get('stack', 'atlas-e2e'))
    ns = os.environ.get('ATLAS_NS') or os.environ.get('ATLAS_E2E_NAMESPACE') or ns_default
    timeout = os.environ.get('ATLAS_E2E_TIMEOUT', '180s')
    if _run(['kubectl', 'get', 'ns', ns], check=False).returncode != 0:
        fallback = ns_default
        if _run(['kubectl', 'get', 'ns', fallback], check=False).returncode == 0:
            ns = fallback

    _run(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/stack/wait_ready.py', ns, timeout])
    _run(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/stack/health_report.py', ns, 'artifacts/ops/stack/health-report.txt'])

    for svc_key in ('minio', 'prometheus', 'grafana'):
        _run(['kubectl', '-n', ns, 'get', 'svc', _svc(root, svc_key)])
        _run(['kubectl', '-n', ns, 'wait', '--for=condition=available', f"deploy/{_svc(root, svc_key)}", f'--timeout={timeout}'])
    for opt in ('redis', 'otel'):
        name = _svc(root, opt)
        if _run(['kubectl', '-n', ns, 'get', f'deploy/{name}'], check=False).returncode == 0:
            _run(['kubectl', '-n', ns, 'wait', '--for=condition=available', f'deploy/{name}', f'--timeout={timeout}'])
    if _run(['kubectl', '-n', ns, 'get', 'deploy/toxiproxy'], check=False).returncode == 0:
        _run(['kubectl', '-n', ns, 'wait', '--for=condition=available', 'deploy/toxiproxy', f'--timeout={timeout}'])

    print('stack-only smoke passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations

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


def _run(cmd: list[str], cwd: Path, env: dict[str, str] | None = None) -> None:
    proc = subprocess.run(cmd, cwd=cwd, env=env, text=True, capture_output=True, check=False)
    if proc.stdout:
        print(proc.stdout, end='')
    if proc.returncode != 0:
        if proc.stderr:
            print(proc.stderr, end='', file=sys.stderr)
        raise SystemExit(proc.returncode)


def main() -> int:
    root = _repo_root()
    stack_ns = os.environ.get('ATLAS_STACK_NAMESPACE') or os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    cluster_name = os.environ.get('ATLAS_E2E_CLUSTER_NAME', 'bijux-atlas-e2e')
    enable_redis = os.environ.get('ATLAS_E2E_ENABLE_REDIS', '0')
    enable_otel = os.environ.get('ATLAS_E2E_ENABLE_OTEL', '1')
    enable_toxiproxy = os.environ.get('ATLAS_E2E_ENABLE_TOXIPROXY', '0')
    timeout = os.environ.get('ATLAS_E2E_TIMEOUT', '180s')
    if os.environ.get('OPS_DRY_RUN', '0') == '1':
        print(f'DRY-RUN install.py cluster={cluster_name} ns={stack_ns}')
        return 0

    clusters = subprocess.run(['kind', 'get', 'clusters'], cwd=root, text=True, capture_output=True, check=False)
    if clusters.returncode != 0 or cluster_name not in clusters.stdout.splitlines():
        _run(['kind', 'create', 'cluster', '--config', str(root / 'ops/stack/kind/cluster.yaml'), '--name', cluster_name], root)

    # Best-effort namespace create, preserving old script semantics.
    subprocess.run(['kubectl', 'get', 'ns', stack_ns], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if subprocess.run(['kubectl', 'get', 'ns', stack_ns], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0:
        subprocess.run(['kubectl', 'create', 'ns', stack_ns], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    for rel in [
        'ops/stack/minio/minio.yaml',
        'ops/stack/prometheus/prometheus.yaml',
        'ops/stack/grafana/grafana.yaml',
    ]:
        _run(['kubectl', 'apply', '-f', str(root / rel)], root)
    if enable_redis == '1':
        _run(['kubectl', 'apply', '-f', str(root / 'ops/stack/redis/redis.yaml')], root)
    if enable_otel == '1':
        _run(['kubectl', 'apply', '-f', str(root / 'ops/stack/otel/otel-collector.yaml')], root)
    if enable_toxiproxy == '1':
        _run(['kubectl', 'apply', '-f', str(root / 'ops/stack/toxiproxy/toxiproxy.yaml')], root)
        _run(['python3', str(root / 'packages/atlasctl/src/atlasctl/commands/ops/stack/toxiproxy/bootstrap.py')], root)

    env = dict(os.environ)
    env['NS'] = stack_ns
    _run(['python3', str(root / 'packages/atlasctl/src/atlasctl/commands/ops/stack/minio/bootstrap.py')], root, env=env)
    _run(['python3', 'packages/atlasctl/src/atlasctl/commands/ops/stack/wait_ready.py', stack_ns, timeout], root)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

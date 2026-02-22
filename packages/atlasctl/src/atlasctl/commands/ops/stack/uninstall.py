#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    ns = os.environ.get('ATLAS_NS', 'atlas-e2e')
    cluster_name = os.environ.get('ATLAS_E2E_CLUSTER_NAME', 'bijux-atlas-e2e')
    for rel in [
        'ops/stack/toxiproxy/toxiproxy.yaml',
        'ops/stack/redis/redis.yaml',
        'ops/stack/otel/otel-collector.yaml',
        'ops/stack/grafana/grafana.yaml',
        'ops/stack/prometheus/prometheus.yaml',
        'ops/stack/minio/minio.yaml',
    ]:
        subprocess.run(['kubectl', 'delete', '-f', str(root / rel), '--ignore-not-found'], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(['kubectl', '-n', ns, 'delete', 'pod', 'minio-bootstrap', '--ignore-not-found'], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(['kubectl', 'delete', 'ns', ns, '--ignore-not-found'], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        clusters = subprocess.run(['kind', 'get', 'clusters'], cwd=root, text=True, capture_output=True, check=False)
        if cluster_name in clusters.stdout.splitlines():
            subprocess.run(['kind', 'delete', 'cluster', '--name', cluster_name], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except FileNotFoundError:
        pass
    print('stack uninstalled')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

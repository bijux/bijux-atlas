from __future__ import annotations

import os
import shutil
import subprocess


def main() -> int:
    root = os.getcwd()
    store_root = os.environ.get('ATLAS_E2E_STORE_ROOT', f'{root}/artifacts/e2e-store')
    out_root = os.environ.get('ATLAS_E2E_OUTPUT_ROOT', f'{root}/artifacts/e2e-datasets')
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    bucket = os.environ.get('MINIO_BUCKET', 'atlas-artifacts')

    shutil.rmtree(store_root, ignore_errors=True)
    shutil.rmtree(out_root, ignore_errors=True)
    os.makedirs(store_root, exist_ok=True)
    os.makedirs(out_root, exist_ok=True)

    if subprocess.call(['kubectl', '-n', ns, 'get', 'svc', 'minio'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
        cmd = (
            'mc alias set local http://minio.{ns}.svc.cluster.local:9000 minioadmin minioadmin\n'
            'mc rm -r --force local/{bucket} || true\n'
            'mc mb --ignore-existing local/{bucket}\n'
        ).format(ns=ns, bucket=bucket)
        subprocess.call([
            'kubectl', '-n', ns, 'run', 'minio-clean',
            '--image=minio/mc:RELEASE.2025-01-17T23-25-50Z',
            '--restart=Never', '--rm', '-i', '--command', '--',
            '/bin/sh', '-ceu', cmd,
        ])

    print('store cleaned')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

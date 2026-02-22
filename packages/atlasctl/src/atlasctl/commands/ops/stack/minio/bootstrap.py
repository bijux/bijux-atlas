from __future__ import annotations

import os
import subprocess
import time


def _run(cmd: list[str], *, check: bool = True, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, check=check, text=True, env=env)


def main() -> int:
    ns = os.environ.get("NS", "atlas-e2e")
    bucket = os.environ.get("MINIO_BUCKET", "atlas-artifacts")
    user = os.environ.get("MINIO_ROOT_USER", "minioadmin")
    password = os.environ.get("MINIO_ROOT_PASSWORD", "minioadmin")
    endpoint = os.environ.get("MINIO_ENDPOINT", f"http://minio.{ns}.svc.cluster.local:9000")

    subprocess.run(["kubectl", "-n", ns, "delete", "pod", "minio-bootstrap", "--ignore-not-found"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(10):
        if subprocess.run(["kubectl", "-n", ns, "get", "pod", "minio-bootstrap"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0:
            break
        time.sleep(1)
    for _ in range(10):
        if subprocess.run(["kubectl", "-n", ns, "get", "serviceaccount", "default"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
            break
        time.sleep(1)
    _run(["kubectl", "-n", ns, "get", "serviceaccount", "default"])
    _run(["kubectl", "-n", ns, "wait", "--for=condition=available", "deploy/minio", f"--timeout={os.environ.get('MINIO_WAIT_TIMEOUT', '300s')}"], check=True)

    shell = (
        f"mc alias set local '{endpoint}' '{user}' '{password}'\n"
        f"mc mb --ignore-existing local/{bucket}\n"
        f"mc anonymous set download local/{bucket}\n"
        f"mc ilm export local/{bucket} >/dev/null 2>&1 || true\n"
    )
    _run(
        [
            "kubectl",
            "-n",
            ns,
            "run",
            "minio-bootstrap",
            "--image=minio/mc:RELEASE.2025-01-17T23-25-50Z",
            "--restart=Never",
            "--rm",
            "-i",
            "--command",
            "--",
            "/bin/sh",
            "-ceu",
            shell,
        ]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

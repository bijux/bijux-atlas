from __future__ import annotations

import os
import subprocess


def main() -> int:
    ns = os.environ.get("ATLAS_E2E_NAMESPACE", "atlas-e2e")
    new_user = "minioadmin"
    new_pass = "minioadmin-rotated"

    subprocess.run(["kubectl", "-n", ns, "get", "secret", "minio-auth"], check=True, stdout=subprocess.DEVNULL)
    patch = f'{{"stringData":{{"MINIO_ROOT_USER":"{new_user}","MINIO_ROOT_PASSWORD":"{new_pass}"}}}}'
    subprocess.run(["kubectl", "-n", ns, "patch", "secret", "minio-auth", "--type", "merge", "-p", patch], check=True, stdout=subprocess.DEVNULL)
    subprocess.run(["kubectl", "-n", ns, "rollout", "restart", "deploy/minio"], check=True, stdout=subprocess.DEVNULL)
    subprocess.run(
        ["kubectl", "-n", ns, "rollout", "status", "deploy/minio", f"--timeout={os.environ.get('OPS_WAIT_TIMEOUT', '180s')}"],
        check=True,
        stdout=subprocess.DEVNULL,
    )
    subprocess.run(["kubectl", "-n", ns, "delete", "pod", "minio-creds-verify", "--ignore-not-found"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    verify = (
        f"mc alias set local 'http://minio.{ns}.svc.cluster.local:9000' '{new_user}' '{new_pass}'; "
        "mc admin info local >/dev/null"
    )
    subprocess.run(
        [
            "kubectl",
            "-n",
            ns,
            "run",
            "minio-creds-verify",
            "--image=minio/mc:RELEASE.2025-01-17T23-25-50Z",
            "--restart=Never",
            "--rm",
            "-i",
            "--command",
            "--",
            "/bin/sh",
            "-ceu",
            verify,
        ],
        check=True,
    )
    print("minio credentials rotation drill passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

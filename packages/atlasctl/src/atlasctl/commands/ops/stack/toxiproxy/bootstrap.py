from __future__ import annotations

import os
import subprocess


def main() -> int:
    ns = os.environ.get("NS", "atlas-e2e")
    shell = (
        f'api="http://toxiproxy.{ns}.svc.cluster.local:8474"\n'
        'curl -fsS -X DELETE "$api/proxies/minio" >/dev/null 2>&1 || true\n'
        f'curl -fsS -X POST "$api/proxies" -H "Content-Type: application/json" -d \'{{"name":"minio","listen":"0.0.0.0:19000","upstream":"minio.{ns}.svc.cluster.local:9000"}}\' >/dev/null\n'
    )
    subprocess.run(
        [
            "kubectl",
            "-n",
            ns,
            "run",
            "toxiproxy-bootstrap",
            "--restart=Never",
            "--rm",
            "-i",
            "--image=curlimages/curl:8.10.1",
            "--command",
            "--",
            "sh",
            "-ceu",
            shell,
        ],
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

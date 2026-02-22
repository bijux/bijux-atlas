from __future__ import annotations

import os
import subprocess
import sys


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    ns = os.environ.get("ATLAS_NS", "atlas-e2e")
    rate_kbps = args[0] if args else "128"
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN toxiproxy bandwidth ns={ns} rate_kbps={rate_kbps}")
        return 0
    script = (
        f'api="http://toxiproxy.{ns}.svc.cluster.local:8474"\n'
        'curl -fsS -X POST "$api/proxies/minio/toxics" -H "Content-Type: application/json" '
        f'-d \'{{"name":"bandwidth","type":"bandwidth","stream":"downstream","attributes":{{"rate":{rate_kbps}}}}}\' >/dev/null || true\n'
    )
    subprocess.run(
        [
            "kubectl",
            "-n",
            ns,
            "run",
            "toxiproxy-throttle",
            "--restart=Never",
            "--rm",
            "-i",
            "--image=curlimages/curl:8.10.1",
            "--command",
            "--",
            "sh",
            "-ceu",
            script,
        ],
        check=True,
    )
    print(f"toxiproxy bandwidth throttle set: {rate_kbps}kbps")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

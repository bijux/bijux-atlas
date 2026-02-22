from __future__ import annotations

import os
import subprocess
import sys


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    ns = os.environ.get("ATLAS_NS", "atlas-e2e")
    latency_ms = args[0] if len(args) > 0 else "0"
    jitter_ms = args[1] if len(args) > 1 else "0"
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN toxiproxy latency ns={ns} latency_ms={latency_ms} jitter_ms={jitter_ms}")
        return 0
    script = f"""api=\"http://toxiproxy.{ns}.svc.cluster.local:8474\"
# clear old toxics
for t in $(curl -fsS \"$api/proxies/minio\" | sed -n 's/.*\"name\":\"\\([^\"]*\\)\".*/\\1/p'); do :; done
curl -fsS -X POST \"$api/proxies/minio/toxics\" -H \"Content-Type: application/json\" \\
  -d '{{\"name\":\"latency\",\"type\":\"latency\",\"stream\":\"downstream\",\"attributes\":{{\"latency\":{latency_ms},\"jitter\":{jitter_ms}}}}}' >/dev/null || true
"""
    subprocess.run(
        [
            "kubectl",
            "-n",
            ns,
            "run",
            "toxiproxy-latency",
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
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

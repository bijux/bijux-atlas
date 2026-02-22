from __future__ import annotations

import os
import subprocess


def main() -> int:
    ns = os.environ.get("ATLAS_NS", "atlas-e2e")
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN cpu throttle via stress pod in namespace {ns}")
        return 0
    script = "apk add --no-cache stress-ng >/dev/null\nstress-ng --cpu 1 --cpu-load 95 --timeout 45s\n"
    subprocess.run(
        [
            "kubectl",
            "-n",
            ns,
            "run",
            "cpu-throttle",
            "--restart=Never",
            "--rm",
            "-i",
            "--image=alpine:3.20",
            "--limits=cpu=100m,memory=128Mi",
            "--requests=cpu=100m,memory=64Mi",
            "--",
            "sh",
            "-ceu",
            script,
        ],
        check=True,
    )
    print("cpu throttle simulation completed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

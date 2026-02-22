from __future__ import annotations

import os
import subprocess


def main() -> int:
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print("DRY-RUN kubectl apply metrics-server manifests")
        return 0
    subprocess.run(
        ["kubectl", "apply", "-f", "https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml"],
        check=True,
    )
    subprocess.run(
        [
            "kubectl",
            "-n",
            "kube-system",
            "patch",
            "deploy",
            "metrics-server",
            "--type=json",
            "-p=[{\"op\":\"add\",\"path\":\"/spec/template/spec/containers/0/args/-\",\"value\":\"--kubelet-insecure-tls\"}]",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    subprocess.run(["kubectl", "-n", "kube-system", "rollout", "status", "deploy/metrics-server", "--timeout=180s"], check=True)
    print("metrics-server installed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

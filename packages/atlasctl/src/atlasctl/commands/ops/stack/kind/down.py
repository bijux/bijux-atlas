from __future__ import annotations

import os
import subprocess


def main() -> int:
    cluster = os.environ.get("ATLAS_E2E_CLUSTER_NAME", "bijux-atlas-e2e")
    clusters = subprocess.run(["kind", "get", "clusters"], capture_output=True, text=True, check=True).stdout.splitlines()
    if cluster not in clusters:
        print(f"kind cluster not present: {cluster}")
        return 0
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN kind delete cluster --name {cluster}")
        return 0
    subprocess.run(["kind", "delete", "cluster", "--name", cluster], check=True)
    print(f"kind cluster deleted: {cluster}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

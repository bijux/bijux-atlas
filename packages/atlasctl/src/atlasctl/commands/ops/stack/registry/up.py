from __future__ import annotations

import os
import subprocess


def main() -> int:
    name = os.environ.get("ATLAS_KIND_REGISTRY_NAME", "kind-registry")
    port = os.environ.get("ATLAS_KIND_REGISTRY_PORT", "5001")
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN docker run -d -p 127.0.0.1:{port}:5000 --restart=always --name {name} registry:2")
        return 0

    inspect = subprocess.run(["docker", "inspect", "-f", "{{.State.Running}}", name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if inspect.returncode != 0:
        subprocess.run(["docker", "run", "-d", "--restart=always", "-p", f"127.0.0.1:{port}:5000", "--name", name, "registry:2"], check=True, stdout=subprocess.DEVNULL)
    node = f"{os.environ.get('ATLAS_E2E_CLUSTER_NAME', 'bijux-atlas-e2e')}-control-plane"
    subprocess.run(["docker", "network", "connect", "kind", name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    print("local registry up:")
    print(f"- container: {name}")
    print(f"- host endpoint: localhost:{port}")
    print(f"- kind node: {node}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

from __future__ import annotations

import os
import subprocess
import sys


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    ns = os.environ.get("ATLAS_NS", "atlas-e2e")
    mode = args[0] if args else "on"
    if mode == "on":
        yaml = f"""apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: block-minio-egress
  namespace: {ns}
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: bijux-atlas
  policyTypes: [Egress]
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: {ns}
          podSelector:
            matchLabels:
              app: minio
      ports:
        - protocol: TCP
          port: 9000
"""
        subprocess.run(["kubectl", "apply", "-f", "-"], input=yaml, text=True, check=True)
        return 0
    subprocess.run(
        ["kubectl", "-n", ns, "delete", "networkpolicy", "block-minio-egress", "--ignore-not-found"],
        stdout=subprocess.DEVNULL,
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

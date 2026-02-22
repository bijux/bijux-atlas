#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl
wait_ready
kubectl -n "$NS" delete pod egress-probe --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run egress-probe --image=nicolaka/netshoot --restart=Never --command -- sh -ceu '
  nslookup kubernetes.default.svc.cluster.local >/dev/null
  if nc -z -w 2 1.1.1.1 80; then
    echo "forbidden egress unexpectedly allowed" >&2
    exit 1
  fi
  exit 0
'
wait_kubectl_condition pod egress-probe Ready 120s || true
kubectl -n "$NS" logs egress-probe || true
kubectl -n "$NS" delete pod egress-probe --ignore-not-found >/dev/null 2>&1 || true
echo "networkpolicy gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())

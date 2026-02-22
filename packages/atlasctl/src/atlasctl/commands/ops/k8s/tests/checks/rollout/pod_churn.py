#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl
install_chart
wait_ready
kubectl -n "$NS" run churn-load --image=curlimages/curl --restart=Never --command -- sh -ceu '
  for i in $(seq 1 500); do
    curl -fsS http://'"$SERVICE_NAME"':8080/healthz >/dev/null || true
  done
' >/dev/null
for _ in 1 2 3; do
  pod="$(pod_name)"
  kubectl -n "$NS" delete pod "$pod" --wait=false >/dev/null
  sleep 3
  wait_ready
done
echo "pod churn drill passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())

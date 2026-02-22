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
POD="$(pod_name)"
START_RESTARTS="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.status.containerStatuses[0].restartCount}')"
kubectl -n "$NS" run liveness-load --image=curlimages/curl --restart=Never --command -- sh -ceu '
  for i in $(seq 1 600); do
    curl -fsS http://'"$SERVICE_NAME"':8080/healthz >/dev/null || true
  done
'
wait_kubectl_condition pod liveness-load Ready 90s || true
sleep 10
POD2="$(pod_name)"
END_RESTARTS="$(kubectl -n "$NS" get pod "$POD2" -o jsonpath='{.status.containerStatuses[0].restartCount}')"
[ "${END_RESTARTS:-0}" -le "$((START_RESTARTS + 1))" ] || { echo "liveness flapping detected" >&2; exit 1; }

echo "liveness under load gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())

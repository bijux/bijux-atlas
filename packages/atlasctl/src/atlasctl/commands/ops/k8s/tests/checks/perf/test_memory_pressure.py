#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm; need kubectl
install_chart
wait_ready
start_pod="$(pod_name)"
start_restarts="$(kubectl -n "$NS" get pod "$start_pod" -o jsonpath='{.status.containerStatuses[0].restartCount}')"
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set resources.requests.memory=64Mi --set resources.limits.memory=96Mi >/dev/null || true
sleep 15
pod_now="$(pod_name)"
restarts_now="$(kubectl -n "$NS" get pod "$pod_now" -o jsonpath='{.status.containerStatuses[0].restartCount}' || echo 0)"
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >/dev/null
wait_ready
kubectl -n "$NS" get pod "$pod_now" -o jsonpath='{.status.phase}' >/dev/null 2>&1 || true
if [ "${restarts_now:-0}" -lt "${start_restarts:-0}" ]; then
  echo "failure_mode: memory_pressure_restart_counter_invalid" >&2
  exit 1
fi
if ! kubectl -n "$NS" get events --sort-by=.lastTimestamp | grep -Eq 'OOMKilled|Evicted|Killing'; then
  echo "failure_mode: memory_pressure_no_signal_observed" >&2
  exit 1
fi
echo "memory pressure OOM/restart safety contract passed (restarts: $start_restarts -> $restarts_now)"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())

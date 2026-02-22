#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl; need helm
wait_ready
POD="$(pod_name)"
CPU_LIMIT="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.spec.containers[0].resources.limits.cpu}')"
MEM_LIMIT="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.spec.containers[0].resources.limits.memory}')"
[ -n "$CPU_LIMIT" ] && [ -n "$MEM_LIMIT" ] || { echo "missing resource limits" >&2; exit 1; }
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set resources.limits.memory=256Mi --set resources.requests.memory=128Mi >/dev/null
wait_ready
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set resources.limits.memory=64Mi --set resources.requests.memory=64Mi >/dev/null || true
sleep 10
OOM_RESTARTS="$(kubectl -n "$NS" get pod "$(pod_name)" -o jsonpath='{.status.containerStatuses[0].restartCount}' || echo 0)"
if [ "${OOM_RESTARTS:-0}" -gt 0 ]; then
  helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >/dev/null
  wait_ready
fi
echo "resource limits gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())

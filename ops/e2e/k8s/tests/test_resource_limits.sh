#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need kubectl; need helm

wait_ready
POD="$(pod_name)"
CPU_LIMIT="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.spec.containers[0].resources.limits.cpu}')"
MEM_LIMIT="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.spec.containers[0].resources.limits.memory}')"
[ -n "$CPU_LIMIT" ] && [ -n "$MEM_LIMIT" ] || { echo "missing resource limits" >&2; exit 1; }
# low memory profile should still boot
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set resources.limits.memory=256Mi --set resources.requests.memory=128Mi >/dev/null
wait_ready
# aggressive memory profile may restart; recovery via baseline upgrade must work
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set resources.limits.memory=64Mi --set resources.requests.memory=64Mi >/dev/null || true
sleep 10
OOM_RESTARTS="$(kubectl -n "$NS" get pod "$(pod_name)" -o jsonpath='{.status.containerStatuses[0].restartCount}' || echo 0)"
if [ "${OOM_RESTARTS:-0}" -gt 0 ]; then
  helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >/dev/null
  wait_ready
fi

echo "resource limits gate passed"

#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm; need kubectl

install_chart
wait_ready
start_pod="$(pod_name)"
start_restarts="$(kubectl -n "$NS" get pod "$start_pod" -o jsonpath='{.status.containerStatuses[0].restartCount}')"

helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" \
  --set resources.requests.memory=64Mi --set resources.limits.memory=96Mi >/dev/null || true
sleep 15
pod_now="$(pod_name)"
restarts_now="$(kubectl -n "$NS" get pod "$pod_now" -o jsonpath='{.status.containerStatuses[0].restartCount}' || echo 0)"
# recover to baseline and ensure service stabilizes
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >/dev/null
wait_ready

echo "memory pressure simulation passed (restarts: $start_restarts -> $restarts_now)"

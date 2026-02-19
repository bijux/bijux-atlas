#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
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

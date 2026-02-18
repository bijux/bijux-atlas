#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

# Contract: configmap update requires explicit rollout to apply.
install_chart
wait_ready
pod_before="$(pod_name)"

kubectl -n "$NS" patch configmap "$SERVICE_NAME" --type merge -p '{"data":{"ATLAS_REQUEST_TIMEOUT_MS":"5100"}}' >/dev/null
sleep 8
pod_no_rollout="$(pod_name)"
[ "$pod_before" = "$pod_no_rollout" ] || {
  echo "unexpected implicit reload on configmap patch" >&2
  exit 1
}

kubectl -n "$NS" rollout restart deployment/"$SERVICE_NAME" >/dev/null
kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=180s >/dev/null
pod_after="$(pod_name)"
[ "$pod_after" != "$pod_before" ] || {
  echo "configmap update did not apply after explicit restart" >&2
  exit 1
}

echo "configmap update reload contract passed"

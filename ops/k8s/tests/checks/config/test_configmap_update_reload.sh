#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

# Contract: configmap update requires explicit rollout to apply.
install_chart
wait_ready
pod_before="$(pod_name)"
with_port_forward 18081
code_before="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/debug/datasets" || true)"
stop_port_forward
[ "$code_before" = "200" ] || {
  echo "expected /debug/datasets to be enabled before config patch, got HTTP $code_before" >&2
  exit 1
}

kubectl -n "$NS" patch configmap "${SERVICE_NAME}-config" --type merge -p '{"data":{"ATLAS_ENABLE_DEBUG_DATASETS":"false"}}' >/dev/null
sleep 8
pod_no_rollout="$(pod_name)"
[ "$pod_before" = "$pod_no_rollout" ] || {
  echo "unexpected implicit reload on configmap patch" >&2
  exit 1
}
with_port_forward 18081
code_no_rollout="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/debug/datasets" || true)"
stop_port_forward
[ "$code_no_rollout" = "200" ] || {
  echo "unexpected behavior change before restart; /debug/datasets returned HTTP $code_no_rollout" >&2
  exit 1
}

kubectl -n "$NS" rollout restart deployment/"$SERVICE_NAME" >/dev/null
kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=180s >/dev/null
pod_after="$(pod_name)"
[ "$pod_after" != "$pod_before" ] || {
  echo "configmap update did not apply after explicit restart" >&2
  exit 1
}
with_port_forward 18081
code_after="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/debug/datasets" || true)"
stop_port_forward
[ "$code_after" = "404" ] || {
  echo "expected /debug/datasets to be disabled after restart, got HTTP $code_after" >&2
  exit 1
}

echo "configmap update reload contract passed"

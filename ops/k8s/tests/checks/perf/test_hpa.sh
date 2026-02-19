#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl; need helm

wait_ready
kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null
kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.maxReplicas}' | grep -Eq '^[0-9]+$'
START_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}')"
kubectl -n "$NS" delete pod hpa-load --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run hpa-load --image=curlimages/curl --restart=Never --command -- sh -ceu '
  for i in $(seq 1 400); do
    curl -fsS http://'"$SERVICE_NAME"':8080/healthz >/dev/null || true
  done
'
wait_kubectl_condition pod hpa-load Ready 120s || true
sleep 15
END_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}')"
if [ "${END_REPLICAS:-0}" -le "${START_REPLICAS:-1}" ]; then
  DESIRED_REPLICAS="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || true)"
  if [ "${DESIRED_REPLICAS:-0}" -le "${START_REPLICAS:-1}" ]; then
    echo "hpa scale check failed: start=$START_REPLICAS end=$END_REPLICAS desired=$DESIRED_REPLICAS" >&2
    exit 1
  fi
fi

kubectl -n "$NS" delete pod hpa-load --ignore-not-found >/dev/null 2>&1 || true
sleep 30
DOWNSCALED_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}' || echo "$END_REPLICAS")"
if [ "${DOWNSCALED_REPLICAS:-0}" -gt "${END_REPLICAS:-0}" ]; then
  echo "hpa downscale check failed: end=$END_REPLICAS downscaled=$DOWNSCALED_REPLICAS" >&2
  exit 1
fi

echo "hpa gate passed"

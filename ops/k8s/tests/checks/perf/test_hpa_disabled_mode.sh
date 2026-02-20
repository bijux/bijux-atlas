#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

wait_ready
if kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null 2>&1; then
  echo "hpa disabled mode check failed: unexpected HPA object ${SERVICE_NAME}" >&2
  exit 1
fi

START_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}')"
kubectl -n "$NS" delete pod hpa-disabled-load --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run hpa-disabled-load --image=curlimages/curl --restart=Never --command -- sh -ceu '
  for i in $(seq 1 300); do
    curl -fsS http://'"$SERVICE_NAME"':8080/v1/datasets?limit=1 >/dev/null || true
    curl -fsS http://'"$SERVICE_NAME"':8080/healthz >/dev/null || true
  done
'
sleep 20
END_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}' || true)"
kubectl -n "$NS" delete pod hpa-disabled-load --ignore-not-found >/dev/null 2>&1 || true

if [ "${END_REPLICAS:-0}" != "${START_REPLICAS:-0}" ]; then
  echo "hpa disabled mode check failed: replicas changed start=${START_REPLICAS} end=${END_REPLICAS}" >&2
  exit 1
fi

echo "hpa disabled mode check passed"

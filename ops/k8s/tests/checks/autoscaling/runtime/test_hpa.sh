#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need kubectl; need helm

ARTIFACT_DIR="${ATLAS_E2E_ARTIFACT_DIR:-artifacts/ops/k8s/hpa}"
mkdir -p "$ARTIFACT_DIR"

dump_hpa_debug() {
  local stamp
  stamp="$(date +%Y%m%d-%H%M%S)"
  kubectl -n "$NS" get hpa "$SERVICE_NAME" -o yaml >"$ARTIFACT_DIR/hpa-${SERVICE_NAME}-${stamp}.yaml" 2>/dev/null || true
  kubectl -n "$NS" describe hpa "$SERVICE_NAME" >"$ARTIFACT_DIR/hpa-${SERVICE_NAME}-${stamp}.describe.txt" 2>/dev/null || true
  kubectl -n "$NS" get events --sort-by=.lastTimestamp >"$ARTIFACT_DIR/hpa-${SERVICE_NAME}-${stamp}.events.txt" 2>/dev/null || true
}

trap 'dump_hpa_debug' ERR

wait_ready
if ! kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null 2>&1; then
  echo "hpa not configured for ${SERVICE_NAME}; skipping hpa gate"
  exit 0
fi

"$SCRIPT_DIR/../contracts/test_metrics_pipeline_ready.sh"

kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.maxReplicas}' | grep -Eq '^[0-9]+$'
START_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}')"
kubectl -n "$NS" delete pod hpa-load --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" run hpa-load --image=curlimages/curl --restart=Never --command -- sh -ceu '
  for i in $(seq 1 800); do
    curl -fsS "http://'"$SERVICE_NAME"':8080/v1/datasets?limit=20" >/dev/null || true
    curl -fsS "http://'"$SERVICE_NAME"':8080/v1/datasets?limit=1" >/dev/null || true
    curl -fsS "http://'"$SERVICE_NAME"':8080/healthz" >/dev/null || true
  done
'
wait_kubectl_condition pod hpa-load Ready 120s || true

if [ "${ATLAS_E2E_HPA_CPU_BURN:-0}" = "1" ]; then
  CPU_BURN_POD="$(kubectl -n "$NS" get pod -l app.kubernetes.io/instance="$RELEASE" -o jsonpath='{.items[0].metadata.name}')"
  kubectl -n "$NS" exec "$CPU_BURN_POD" -- sh -ceu 'end=$(( $(date +%s) + 90 )); while [ "$(date +%s)" -lt "$end" ]; do :; done' >/dev/null 2>&1 &
  CPU_BURN_PID=$!
fi

if ! kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.currentMetrics}' | grep -q '[^[:space:]]'; then
  echo "hpa scale check failed: .status.currentMetrics is empty" >&2
  exit 1
fi

DESIRED_BASE="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo "${START_REPLICAS}")"
DESIRED_CHANGED=0
for _ in $(seq 1 30); do
  DESIRED_NOW="$(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.status.desiredReplicas}' || echo "${DESIRED_BASE}")"
  if [ "${DESIRED_NOW:-0}" -gt "${DESIRED_BASE:-0}" ]; then
    DESIRED_CHANGED=1
    break
  fi
  sleep 5
done

END_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}' || echo "${START_REPLICAS}")"
if [ "$DESIRED_CHANGED" -ne 1 ] && [ "${END_REPLICAS:-0}" -le "${START_REPLICAS:-1}" ]; then
  echo "hpa scale check failed: desiredReplicas did not increase and replicas did not scale up (start=$START_REPLICAS end=$END_REPLICAS desired_base=$DESIRED_BASE)" >&2
  exit 1
fi

kubectl -n "$NS" delete pod hpa-load --ignore-not-found >/dev/null 2>&1 || true
if [ -n "${CPU_BURN_PID:-}" ]; then
  wait "$CPU_BURN_PID" >/dev/null 2>&1 || true
fi

DOWNSCALED_REPLICAS="$END_REPLICAS"
for _ in $(seq 1 30); do
  DOWNSCALED_REPLICAS="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.replicas}' || echo "$END_REPLICAS")"
  if [ "${DOWNSCALED_REPLICAS:-0}" -lt "${END_REPLICAS:-0}" ]; then
    break
  fi
  sleep 5
done
if [ "${DOWNSCALED_REPLICAS:-0}" -ge "${END_REPLICAS:-0}" ] && [ "${END_REPLICAS:-0}" -gt "${START_REPLICAS:-0}" ]; then
  echo "hpa downscale check failed: no downscale observed in bounded window (scaled_end=$END_REPLICAS current=$DOWNSCALED_REPLICAS)" >&2
  exit 1
fi

echo "hpa gate passed"

#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

if ! kubectl get --raw /apis/metrics.k8s.io/v1beta1 >/dev/null 2>&1; then
  echo "metrics pipeline preflight failed: metrics.k8s.io API is not reachable (metrics-server missing/unhealthy)" >&2
  exit 1
fi

if ! kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null 2>&1; then
  echo "metrics pipeline preflight skipped: no HPA object found for ${SERVICE_NAME}"
  exit 0
fi

if ! kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.metrics[*].pods.metric.name}' | grep -q '[^[:space:]]'; then
  echo "metrics pipeline preflight passed: HPA uses only resource metrics"
  exit 0
fi

if ! kubectl get --raw /apis/custom.metrics.k8s.io/v1beta1 >/tmp/atlas-custom-metrics-api.json 2>/dev/null; then
  echo "metrics pipeline preflight failed: custom.metrics.k8s.io API is not reachable" >&2
  exit 1
fi

for metric in $(kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.metrics[*].pods.metric.name}'); do
  if ! grep -q "\"name\"[[:space:]]*:[[:space:]]*\"pods/${metric}\"" /tmp/atlas-custom-metrics-api.json; then
    echo "metrics pipeline preflight failed: required custom metric not advertised by adapter: ${metric}" >&2
    exit 1
  fi
done

echo "metrics pipeline preflight passed"

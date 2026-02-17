#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
MODE="${ATLAS_OBS_MODE:-minimal}"

kubectl apply -f "$ROOT/ops/stack/prometheus/prometheus.yaml"
kubectl apply -f "$ROOT/ops/stack/otel/otel-collector.yaml"

if kubectl api-resources | grep -q "^prometheusrules"; then
  kubectl apply -f "$ROOT/ops/observability/alerts/atlas-alert-rules.yaml"
else
  echo "PrometheusRule CRD not present; running in minimal mode"
fi

if [ "$MODE" = "full" ]; then
  if ! kubectl api-resources | grep -q "^servicemonitors"; then
    echo "full mode requested but ServiceMonitor CRD missing" >&2
    exit 1
  fi
fi

echo "observability pack installed (mode=$MODE)"

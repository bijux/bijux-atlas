#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
PROFILE="${ATLAS_OBS_PROFILE:-kind}"

kubectl apply -f "$ROOT/ops/stack/prometheus/prometheus.yaml"
kubectl apply -f "$ROOT/ops/stack/otel/otel-collector.yaml"

if kubectl api-resources | grep -q "^prometheusrules"; then
  kubectl apply -f "$ROOT/ops/observability/alerts/atlas-alert-rules.yaml"
else
  echo "PrometheusRule CRD not present; running in kind profile"
fi

if [ "$PROFILE" = "cluster" ]; then
  if ! kubectl api-resources | grep -q "^servicemonitors"; then
    echo "cluster profile requested but ServiceMonitor CRD missing" >&2
    exit 1
  fi
fi

echo "observability pack installed (profile=$PROFILE)"

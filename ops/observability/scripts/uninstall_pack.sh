#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
kubectl delete -f "$ROOT/ops/stack/prometheus/prometheus.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete -f "$ROOT/ops/stack/otel/otel-collector.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete -f "$ROOT/ops/observability/alerts/atlas-alert-rules.yaml" --ignore-not-found >/dev/null 2>&1 || true
echo "observability pack uninstalled"

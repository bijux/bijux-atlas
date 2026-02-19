#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
source "$ROOT/_lib/common.sh"
NS="${ATLAS_NS:-${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_stack)}}"
TIMEOUT="${ATLAS_E2E_TIMEOUT:-180s}"

if ! kubectl get ns "$NS" >/dev/null 2>&1; then
  fallback_ns="$(ops_layer_ns_stack)"
  if kubectl get ns "$fallback_ns" >/dev/null 2>&1; then
    NS="$fallback_ns"
  fi
fi

"$ROOT/stack/scripts/wait_ready.sh" "$NS" "$TIMEOUT"
"$ROOT/stack/scripts/health_report.sh" "$NS" "artifacts/ops/stack/health-report.txt" >/dev/null

kubectl -n "$NS" get svc "$(ops_layer_contract_get services.minio.service_name)" >/dev/null
kubectl -n "$NS" get svc "$(ops_layer_contract_get services.prometheus.service_name)" >/dev/null
kubectl -n "$NS" get svc "$(ops_layer_contract_get services.grafana.service_name)" >/dev/null

kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.minio.service_name)" --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.prometheus.service_name)" --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.grafana.service_name)" --timeout="$TIMEOUT" >/dev/null

if kubectl -n "$NS" get deploy/"$(ops_layer_contract_get services.redis.service_name)" >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.redis.service_name)" --timeout="$TIMEOUT" >/dev/null
fi
if kubectl -n "$NS" get deploy/"$(ops_layer_contract_get services.otel.service_name)" >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=available deploy/"$(ops_layer_contract_get services.otel.service_name)" --timeout="$TIMEOUT" >/dev/null
fi
if kubectl -n "$NS" get deploy/toxiproxy >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=available deploy/toxiproxy --timeout="$TIMEOUT" >/dev/null
fi

echo "stack-only smoke passed"

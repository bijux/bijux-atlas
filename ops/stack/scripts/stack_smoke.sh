#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
NS="${ATLAS_NS:-${ATLAS_E2E_NAMESPACE:-atlas-e2e}}"
TIMEOUT="${ATLAS_E2E_TIMEOUT:-180s}"

if ! kubectl get ns "$NS" >/dev/null 2>&1; then
  if kubectl get ns atlas-e2e >/dev/null 2>&1; then
    NS="atlas-e2e"
  fi
fi

"$ROOT/stack/scripts/wait_ready.sh" "$NS" "$TIMEOUT"
"$ROOT/stack/scripts/health_report.sh" "$NS" "artifacts/ops/stack/health-report.txt" >/dev/null

kubectl -n "$NS" get svc minio >/dev/null
kubectl -n "$NS" get svc prometheus >/dev/null
kubectl -n "$NS" get svc grafana >/dev/null

kubectl -n "$NS" wait --for=condition=available deploy/minio --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/prometheus --timeout="$TIMEOUT" >/dev/null
kubectl -n "$NS" wait --for=condition=available deploy/grafana --timeout="$TIMEOUT" >/dev/null

if kubectl -n "$NS" get deploy/redis >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=available deploy/redis --timeout="$TIMEOUT" >/dev/null
fi
if kubectl -n "$NS" get deploy/otel-collector >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=available deploy/otel-collector --timeout="$TIMEOUT" >/dev/null
fi
if kubectl -n "$NS" get deploy/toxiproxy >/dev/null 2>&1; then
  kubectl -n "$NS" wait --for=condition=available deploy/toxiproxy --timeout="$TIMEOUT" >/dev/null
fi

echo "stack-only smoke passed"

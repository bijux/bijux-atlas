#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
NS="${ATLAS_NS:-atlas-e2e}"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"

kubectl delete -f "$ROOT/stack/toxiproxy/toxiproxy.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete -f "$ROOT/stack/redis/redis.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete -f "$ROOT/stack/otel/otel-collector.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete -f "$ROOT/stack/prometheus/prometheus.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl delete -f "$ROOT/stack/minio/minio.yaml" --ignore-not-found >/dev/null 2>&1 || true
kubectl -n "$NS" delete pod minio-bootstrap --ignore-not-found >/dev/null 2>&1 || true
kubectl delete ns "$NS" --ignore-not-found >/dev/null 2>&1 || true

if command -v kind >/dev/null 2>&1 && kind get clusters | grep -qx "$CLUSTER_NAME"; then
  kind delete cluster --name "$CLUSTER_NAME" >/dev/null 2>&1 || true
fi

echo "stack uninstalled"

#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
ENABLE_REDIS="${ATLAS_E2E_ENABLE_REDIS:-0}"
ENABLE_OTEL="${ATLAS_E2E_ENABLE_OTEL:-0}"

if ! command -v kind >/dev/null 2>&1; then
  echo "kind is required" >&2
  exit 1
fi
if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi

if ! kind get clusters | grep -qx "$CLUSTER_NAME"; then
  kind create cluster --config "$ROOT/ops/e2e/stack/kind/cluster.yaml" --name "$CLUSTER_NAME"
fi

kubectl apply -f "$ROOT/ops/e2e/stack/minio/minio.yaml"
kubectl apply -f "$ROOT/ops/observability/prometheus/prometheus.yaml"

if [ "$ENABLE_REDIS" = "1" ]; then
  kubectl apply -f "$ROOT/ops/e2e/stack/redis/redis.yaml"
fi

if [ "$ENABLE_OTEL" = "1" ]; then
  kubectl apply -f "$ROOT/ops/observability/otel/otel-collector.yaml"
fi

"$ROOT/ops/e2e/stack/minio/bootstrap.sh"

echo "e2e stack is up"

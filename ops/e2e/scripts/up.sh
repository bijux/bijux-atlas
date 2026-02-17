#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
ENABLE_REDIS="${ATLAS_E2E_ENABLE_REDIS:-0}"
ENABLE_OTEL="${ATLAS_E2E_ENABLE_OTEL:-0}"
ENABLE_TOXIPROXY="${ATLAS_E2E_ENABLE_TOXIPROXY:-0}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"

if ! command -v kind >/dev/null 2>&1; then
  echo "kind is required" >&2
  exit 1
fi
if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi

if ! kind get clusters | grep -qx "$CLUSTER_NAME"; then
  kind create cluster --config "$ROOT/ops/stack/kind/cluster.yaml" --name "$CLUSTER_NAME"
fi

if kubectl get ns "$NS" >/dev/null 2>&1; then
  if [ -n "$(kubectl get ns "$NS" -o jsonpath='{.metadata.deletionTimestamp}' 2>/dev/null)" ]; then
    echo "namespace $NS is terminating; waiting for deletion to complete..."
    for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24; do
      if ! kubectl get ns "$NS" >/dev/null 2>&1; then
        break
      fi
      sleep 5
    done
    if kubectl get ns "$NS" >/dev/null 2>&1 && [ -n "$(kubectl get ns "$NS" -o jsonpath='{.metadata.deletionTimestamp}' 2>/dev/null)" ]; then
      echo "namespace $NS is still terminating after timeout" >&2
      exit 1
    fi
  fi
fi
kubectl get ns "$NS" >/dev/null 2>&1 || kubectl create ns "$NS" >/dev/null

kubectl apply -f "$ROOT/ops/stack/minio/minio.yaml"
kubectl apply -f "$ROOT/ops/stack/prometheus/prometheus.yaml"

if [ "$ENABLE_REDIS" = "1" ]; then
  kubectl apply -f "$ROOT/ops/stack/redis/redis.yaml"
fi

if [ "$ENABLE_OTEL" = "1" ]; then
  kubectl apply -f "$ROOT/ops/stack/otel/otel-collector.yaml"
fi

if [ "$ENABLE_TOXIPROXY" = "1" ]; then
  kubectl apply -f "$ROOT/ops/stack/toxiproxy/toxiproxy.yaml"
  "$ROOT/ops/stack/toxiproxy/bootstrap.sh"
fi

"$ROOT/ops/stack/minio/bootstrap.sh"
"$ROOT/ops/stack/scripts/wait_ready.sh" "$NS" "${ATLAS_E2E_TIMEOUT:-180s}"

echo "e2e stack is up"

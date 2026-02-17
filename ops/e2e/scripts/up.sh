#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"
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

if ! ops_wait_namespace_termination "$NS" 120; then
  echo "namespace $NS is still terminating after timeout" >&2
  ops_kubectl_dump_bundle "$NS" "$(ops_artifact_dir failure-bundle)"
  exit 1
fi
ops_kubectl get ns "$NS" >/dev/null 2>&1 || ops_kubectl create ns "$NS" >/dev/null

ops_kubectl_retry apply -f "$ROOT/ops/stack/minio/minio.yaml"
ops_kubectl_retry apply -f "$ROOT/ops/stack/prometheus/prometheus.yaml"

if [ "$ENABLE_REDIS" = "1" ]; then
  ops_kubectl_retry apply -f "$ROOT/ops/stack/redis/redis.yaml"
fi

if [ "$ENABLE_OTEL" = "1" ]; then
  ops_kubectl_retry apply -f "$ROOT/ops/stack/otel/otel-collector.yaml"
fi

if [ "$ENABLE_TOXIPROXY" = "1" ]; then
  ops_kubectl_retry apply -f "$ROOT/ops/stack/toxiproxy/toxiproxy.yaml"
  "$ROOT/ops/stack/toxiproxy/bootstrap.sh"
fi

"$ROOT/ops/stack/minio/bootstrap.sh"
"$ROOT/ops/stack/scripts/wait_ready.sh" "$NS" "${ATLAS_E2E_TIMEOUT:-180s}"

echo "e2e stack is up"

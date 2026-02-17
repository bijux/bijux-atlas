#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
USE_LOCAL_IMAGE="${ATLAS_E2E_USE_LOCAL_IMAGE:-1}"
LOCAL_IMAGE_REF="${ATLAS_E2E_LOCAL_IMAGE:-bijux-atlas:local}"
HELM_WAIT="${ATLAS_E2E_HELM_WAIT:-0}"
HELM_TIMEOUT="${ATLAS_E2E_HELM_TIMEOUT:-5m}"

if ! command -v helm >/dev/null 2>&1; then
  echo "helm is required" >&2
  exit 1
fi
if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi
if [ "$USE_LOCAL_IMAGE" = "1" ]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "docker is required when ATLAS_E2E_USE_LOCAL_IMAGE=1" >&2
    exit 1
  fi
  if ! command -v kind >/dev/null 2>&1; then
    echo "kind is required when ATLAS_E2E_USE_LOCAL_IMAGE=1" >&2
    exit 1
  fi
fi

kubectl get ns "$NS" >/dev/null 2>&1 || kubectl create ns "$NS"

if [ "$USE_LOCAL_IMAGE" = "1" ]; then
  if ! docker image inspect "$LOCAL_IMAGE_REF" >/dev/null 2>&1; then
    docker build -t "$LOCAL_IMAGE_REF" -f "$ROOT/docker/Dockerfile" "$ROOT"
  fi
  kind load docker-image "$LOCAL_IMAGE_REF" --name "$CLUSTER_NAME"
  if [ "$HELM_WAIT" = "1" ]; then
    helm upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
      --namespace "$NS" \
      -f "$VALUES" \
      --wait --timeout "$HELM_TIMEOUT" \
      --set image.repository="${LOCAL_IMAGE_REF%:*}" \
      --set image.tag="${LOCAL_IMAGE_REF#*:}" \
      --set image.pullPolicy=IfNotPresent
  else
    helm upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
      --namespace "$NS" \
      -f "$VALUES" \
      --set image.repository="${LOCAL_IMAGE_REF%:*}" \
      --set image.tag="${LOCAL_IMAGE_REF#*:}" \
      --set image.pullPolicy=IfNotPresent
  fi
else
  if [ "$HELM_WAIT" = "1" ]; then
    helm upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
      --namespace "$NS" \
      -f "$VALUES" \
      --wait --timeout "$HELM_TIMEOUT"
  else
    helm upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
      --namespace "$NS" \
      -f "$VALUES"
  fi
fi

echo "atlas deployed: release=$RELEASE ns=$NS"

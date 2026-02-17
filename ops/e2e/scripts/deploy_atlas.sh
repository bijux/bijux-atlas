#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/e2e/stack/values/local.yaml}"

if ! command -v helm >/dev/null 2>&1; then
  echo "helm is required" >&2
  exit 1
fi
if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi

kubectl get ns "$NS" >/dev/null 2>&1 || kubectl create ns "$NS"

helm upgrade --install "$RELEASE" "$ROOT/charts/bijux-atlas" \
  --namespace "$NS" \
  --wait --timeout 5m \
  -f "$VALUES"

echo "atlas deployed: release=$RELEASE ns=$NS"

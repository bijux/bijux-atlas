#!/usr/bin/env bash
set -euo pipefail

source "$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

NS="${1:-${ATLAS_E2E_NAMESPACE:-atlas-e2e-${USER:-local}}}"
RELEASE="${2:-${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}}"
TS="$(date +%Y%m%d-%H%M%S)"
OUT="$ARTIFACTS_ROOT/k8s-failures/$TS"
ops_mkdir_artifacts
mkdir -p "$OUT"

ops_capture_artifacts "$NS" "$RELEASE" "$OUT"
kubectl -n "$NS" get configmap "$RELEASE" -o yaml > "$OUT/configmap.yaml" 2>/dev/null || true
if kubectl top pods -n "$NS" >/dev/null 2>&1; then
  kubectl top pods -n "$NS" > "$OUT/top-pods.txt" 2>/dev/null || true
fi
tar -czf "$ARTIFACTS_ROOT/k8s-failure-bundle-$TS.tar.gz" -C "$OUT" . >/dev/null 2>&1 || true

echo "k8s test report: $OUT"

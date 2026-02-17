#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
NS="${1:-${ATLAS_E2E_NAMESPACE:-atlas-e2e-${USER:-local}}}"
RELEASE="${2:-${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}}"
TS="$(date +%Y%m%d-%H%M%S)"
OUT="$ROOT/artifacts/ops/k8s-failures/$TS"
mkdir -p "$OUT"

kubectl get ns "$NS" > "$OUT/ns.txt" 2>/dev/null || true
kubectl -n "$NS" get all -o wide > "$OUT/all.txt" 2>/dev/null || true
kubectl -n "$NS" get events --sort-by=.lastTimestamp > "$OUT/events.txt" 2>/dev/null || true
kubectl -n "$NS" logs -l app.kubernetes.io/instance="$RELEASE" --all-containers --tail=2000 > "$OUT/logs.txt" 2>/dev/null || true

echo "k8s test report: $OUT"

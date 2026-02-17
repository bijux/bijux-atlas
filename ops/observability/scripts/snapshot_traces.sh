#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT_DIR="${1:-$ROOT/artifacts/ops/observability}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
mkdir -p "$OUT_DIR"
POD="$(kubectl -n "$NS" get pod -l app=otel-collector -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)"
if [ -z "$POD" ]; then
  touch "$OUT_DIR/traces.snapshot.log"
  echo "otel collector not present; wrote empty $OUT_DIR/traces.snapshot.log"
  exit 0
fi
kubectl -n "$NS" logs "$POD" --tail=1000 > "$OUT_DIR/traces.snapshot.log" || touch "$OUT_DIR/traces.snapshot.log"
echo "wrote $OUT_DIR/traces.snapshot.log"

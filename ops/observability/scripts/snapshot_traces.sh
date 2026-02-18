#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT_DIR="${1:-$ROOT/artifacts/ops/observability}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
mkdir -p "$OUT_DIR"
POD="$(kubectl -n "$NS" get pod -l app=otel-collector -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)"
if [ -z "$POD" ]; then
  NS_DISCOVERED="$(kubectl get pod -A -l app=otel-collector -o jsonpath='{.items[0].metadata.namespace}' 2>/dev/null || true)"
  if [ -n "$NS_DISCOVERED" ]; then
    NS="$NS_DISCOVERED"
    POD="$(kubectl -n "$NS" get pod -l app=otel-collector -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)"
  fi
fi
if [ -z "$POD" ]; then
  touch "$OUT_DIR/traces.snapshot.log"
  touch "$OUT_DIR/traces.exemplars.log"
  echo "otel collector not present; wrote empty $OUT_DIR/traces.snapshot.log"
  exit 0
fi
kubectl -n "$NS" logs "$POD" --tail=1000 > "$OUT_DIR/traces.snapshot.log" || touch "$OUT_DIR/traces.snapshot.log"
grep -Ei 'trace_id|traceid|span_id|spanid|export.*span|otlp|traces?' "$OUT_DIR/traces.snapshot.log" > "$OUT_DIR/traces.exemplars.log" || true
if [ ! -s "$OUT_DIR/traces.exemplars.log" ] && [ -s "$OUT_DIR/traces.snapshot.log" ]; then
  head -n 20 "$OUT_DIR/traces.snapshot.log" > "$OUT_DIR/traces.exemplars.log" || true
fi
echo "wrote $OUT_DIR/traces.snapshot.log"

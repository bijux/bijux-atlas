#!/usr/bin/env bash
set -euo pipefail
NS="${1:-atlas-e2e}"
OUT_DIR="${2:-artifacts/ops/stack/logs}"
mkdir -p "$OUT_DIR"
for p in $(kubectl -n "$NS" get pods -o jsonpath='{.items[*].metadata.name}' 2>/dev/null); do
  kubectl -n "$NS" logs "$p" --tail=2000 > "$OUT_DIR/$p.log" 2>/dev/null || true
done
echo "$OUT_DIR"

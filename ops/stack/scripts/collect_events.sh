#!/usr/bin/env bash
set -euo pipefail
NS="${1:-atlas-e2e}"
OUT="${2:-artifacts/ops/stack/events.txt}"
mkdir -p "$(dirname "$OUT")"
kubectl get events -n "$NS" --sort-by=.lastTimestamp > "$OUT" 2>/dev/null || true
echo "$OUT"

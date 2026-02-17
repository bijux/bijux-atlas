#!/usr/bin/env bash
set -euo pipefail
NS="${1:-atlas-e2e}"
OUT="${2:-artifacts/ops/stack/health-report.txt}"
mkdir -p "$(dirname "$OUT")"
{
  echo "namespace=$NS"
  echo "timestamp=$(date -u +%FT%TZ)"
  echo "--- nodes ---"
  kubectl get nodes -o wide || true
  echo "--- pods ---"
  kubectl -n "$NS" get pods -o wide || true
  echo "--- services ---"
  kubectl -n "$NS" get svc || true
  echo "--- storageclass ---"
  kubectl get storageclass || true
} > "$OUT"
echo "$OUT"

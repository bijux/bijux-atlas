#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/artifacts/perf/results}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
SERVICE="${ATLAS_E2E_SERVICE_NAME:-$RELEASE-bijux-atlas}"

mkdir -p "$OUT_DIR"

if command -v kubectl >/dev/null 2>&1 && kubectl -n "$NS" get deploy "$SERVICE" >/dev/null 2>&1; then
  (
    sleep 5
    kubectl -n "$NS" rollout restart deploy/"$SERVICE" >/dev/null || true
    sleep 5
    kubectl -n "$NS" rollout undo deploy/"$SERVICE" >/dev/null || true
    kubectl -n "$NS" rollout status deploy/"$SERVICE" --timeout=240s >/dev/null || true
  ) &
fi

"$ROOT/ops/load/scripts/run_suite.sh" load-under-rollback.json "$OUT_DIR"

echo "load-under-rollback complete"

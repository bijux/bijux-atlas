#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
LOCAL_PORT="${ATLAS_E2E_LOCAL_PORT:-18080}"
OUT_DIR="${1:-$ROOT/artifacts/ops/obs}"
mkdir -p "$OUT_DIR"
CURL="curl --connect-timeout 2 --max-time 5 -fsS"
if ! $CURL "$ATLAS_BASE_URL/healthz" >/dev/null 2>&1; then
  if ! kubectl config current-context >/dev/null 2>&1; then
    : > "$OUT_DIR/metrics.prom"
    echo "metrics snapshot skipped: kubectl context is not configured"
    echo "wrote $OUT_DIR/metrics.prom"
    exit 0
  fi
  POD="$(kubectl -n "$NS" get pods -l app.kubernetes.io/instance="$RELEASE" --field-selector=status.phase=Running -o name 2>/dev/null | tail -n1 | cut -d/ -f2)"
  if [ -z "$POD" ]; then
    : > "$OUT_DIR/metrics.prom"
    echo "metrics snapshot skipped: no running atlas pod found in namespace '$NS'"
    echo "wrote $OUT_DIR/metrics.prom"
    exit 0
  fi
  kubectl -n "$NS" port-forward "pod/$POD" "$LOCAL_PORT:8080" >/tmp/atlas-snapshot-metrics-port-forward.log 2>&1 &
  PF_PID=$!
  trap 'kill "$PF_PID" >/dev/null 2>&1 || true' EXIT INT TERM
  ATLAS_BASE_URL="http://127.0.0.1:$LOCAL_PORT"
fi
if ! $CURL "$ATLAS_BASE_URL/metrics" > "$OUT_DIR/metrics.prom"; then
  : > "$OUT_DIR/metrics.prom"
fi
cat > "$OUT_DIR/baseline-metadata.json" <<META
{
  "git_sha": "${GIT_SHA:-$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)}",
  "image_digest": "${ATLAS_IMAGE_DIGEST:-unknown}",
  "dataset_hash": "${ATLAS_DATASET_HASH:-unknown}",
  "release": "${ATLAS_RELEASE:-unknown}"
}
META
echo "wrote $OUT_DIR/metrics.prom"

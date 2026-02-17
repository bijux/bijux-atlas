#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
OUT_DIR="${1:-$ROOT/artifacts/ops/observability}"
mkdir -p "$OUT_DIR"
curl -fsS "$ATLAS_BASE_URL/metrics" > "$OUT_DIR/metrics.prom"
cat > "$OUT_DIR/baseline-metadata.json" <<META
{
  "git_sha": "${GIT_SHA:-$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)}",
  "image_digest": "${ATLAS_IMAGE_DIGEST:-unknown}",
  "dataset_hash": "${ATLAS_DATASET_HASH:-unknown}",
  "release": "${ATLAS_RELEASE:-unknown}"
}
META
echo "wrote $OUT_DIR/metrics.prom"

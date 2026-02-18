#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: simulate store outage while load is active and assert degraded-mode signals.
# stability: public
# called-by: make ops-drill-minio-outage, make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

"$ROOT/stack/faults/block-minio.sh" on
trap '"$ROOT/stack/faults/block-minio.sh" off >/dev/null 2>&1 || true' EXIT

"$ROOT/load/scripts/run_suite.sh" store-outage-mid-spike.json artifacts/perf/results || true
curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null
curl -fsS "$ATLAS_BASE_URL/metrics" | grep -E '^bijux_store_breaker_open' >/dev/null

echo "store outage under load drill passed"

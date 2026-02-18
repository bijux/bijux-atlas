#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: exercise cache stampede path and assert singleflight metrics are emitted.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
for _ in $(seq 1 20); do
  curl -fsS "$ATLAS_BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=5" >/dev/null &
done
wait
curl -fsS "$ATLAS_BASE_URL/metrics" | grep -Eq 'bijux_dataset_singleflight|bijux_dataset_waiters|bijux_dataset_cache_misses_total'
echo "cache stampede drill passed"

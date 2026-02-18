#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: print cache hit/miss and local disk usage status.
# stability: public
# called-by: make ops-cache-status
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
metrics="$(curl -fsS "$BASE_URL/metrics" 2>/dev/null || true)"
hits="$(printf '%s\n' "$metrics" | awk '/^bijux_dataset_hits/{print $2}' | tail -n1)"
misses="$(printf '%s\n' "$metrics" | awk '/^bijux_dataset_misses/{print $2}' | tail -n1)"
usage="$(du -sk "$ROOT/artifacts/e2e-store" 2>/dev/null | awk '{print $1*1024}')"
hits="${hits:-0}"
misses="${misses:-0}"
usage="${usage:-0}"
total=$((hits + misses))
if [ "$total" -gt 0 ]; then
  ratio="$(awk "BEGIN { printf \"%.6f\", $hits / $total }")"
else
  ratio="0.000000"
fi
echo "cache_hits=$hits"
echo "cache_misses=$misses"
echo "cache_hit_ratio=$ratio"
echo "cache_disk_bytes=$usage"
mkdir -p "$ROOT/artifacts/ops"
cat > "$ROOT/artifacts/ops/cache-status.json" <<JSON
{"cache_hits":$hits,"cache_misses":$misses,"cache_hit_ratio":$ratio,"cache_disk_bytes":$usage}
JSON

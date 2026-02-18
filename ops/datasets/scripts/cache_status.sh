#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: print cache hit/miss and local disk usage status.
# stability: public
# called-by: make ops-cache-status
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
metrics="$(curl -fsS "$BASE_URL/metrics" 2>/dev/null || true)"
hits="$(printf '%s\n' "$metrics" | awk '/^bijux_dataset_cache_hit_total/{print $2}' | tail -n1)"
misses="$(printf '%s\n' "$metrics" | awk '/^bijux_dataset_cache_miss_total/{print $2}' | tail -n1)"
usage="$(du -sk "$ROOT/artifacts/e2e-store" 2>/dev/null | awk '{print $1*1024}')"
echo "cache_hits=${hits:-0}"
echo "cache_misses=${misses:-0}"
echo "cache_disk_bytes=${usage:-0}"

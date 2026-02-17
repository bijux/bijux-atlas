#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
TMP="$(mktemp)"
curl -fsS "$ATLAS_BASE_URL/metrics" > "$TMP" || true
grep -Eq 'process_resident_memory_bytes|bijux_dataset_disk_usage_bytes' "$TMP"
rm -f "$TMP"
echo "memory growth drill assertions passed"

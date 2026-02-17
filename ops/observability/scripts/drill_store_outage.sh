#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
TMP="$(mktemp)"
curl -fsS "$ATLAS_BASE_URL/metrics" > "$TMP" || true
grep -Eq 'bijux_store_(circuit|breaker)_open' "$TMP"
grep -q 'bijux_http_requests_total' "$TMP"
grep -q 'bijux_dataset_hits' "$TMP"
rm -f "$TMP"
echo "store outage drill assertions passed"

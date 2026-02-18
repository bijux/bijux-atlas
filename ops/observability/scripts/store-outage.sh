#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

# Cached path should remain available.
curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null
curl -fsS "$ATLAS_BASE_URL/metrics" > "$tmp"

grep -Eq 'bijux_store_(circuit|breaker)_open' "$tmp"
grep -q 'bijux_dataset_hits' "$tmp"

echo "store outage drill assertions passed"

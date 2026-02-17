#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
TMP="$(mktemp)"
curl -fsS "$ATLAS_BASE_URL/metrics" > "$TMP" || true
grep -q 'bijux_overload_shedding_active' "$TMP"
grep -q 'bijux_cheap_queries_served_while_overloaded_total' "$TMP"
rm -f "$TMP"
echo "overload drill assertions passed"

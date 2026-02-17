#!/usr/bin/env bash
set -euo pipefail
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

metrics_before="$(mktemp)"
metrics_after="$(mktemp)"
trap 'rm -f "$metrics_before" "$metrics_after"' EXIT

curl -fsS "$ATLAS_BASE_URL/metrics" > "$metrics_before"
# Fire a heavy-ish request likely to trigger policy rejection under strict limits.
code="$(curl -s -o /dev/null -w '%{http_code}' "$ATLAS_BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-999999999&limit=500")"

curl -fsS "$ATLAS_BASE_URL/metrics" > "$metrics_after"

grep -q 'bijux_overload_shedding_active' "$metrics_after"
grep -q 'bijux_cheap_queries_served_while_overloaded_total' "$metrics_after"

case "$code" in
  200|422|429|503) ;;
  *) echo "unexpected overload drill status code: $code" >&2; exit 1 ;;
esac

echo "overload drill assertions passed"

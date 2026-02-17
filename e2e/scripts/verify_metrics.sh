#!/usr/bin/env sh
set -eu

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
METRICS="$(curl -fsS "$BASE_URL/metrics")"

required='\
bijux_http_requests_total\
bijux_http_request_latency_p95_seconds\
bijux_dataset_hits\
bijux_dataset_misses\
bijux_store_download_p95_seconds\
bijux_sqlite_query_latency_p95_seconds\
bijux_overload_shedding_active\
bijux_store_breaker_open\
bijux_errors_total\
'

for m in $required; do
  echo "$METRICS" | grep -q "^$m" || { echo "missing metric: $m" >&2; exit 1; }
done

nonzero='\
bijux_http_requests_total\
bijux_dataset_hits\
'
for m in $nonzero; do
  v="$(echo "$METRICS" | awk -v k="$m" '$1==k {print $2}' | head -n1)"
  [ -n "${v:-}" ] || { echo "metric missing value: $m" >&2; exit 1; }
  awk -v n="$v" 'BEGIN {exit !(n+0 > 0)}'
done

echo "metrics verified"

#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
QUERY="${QUERY:-/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38}"
OUT_DIR="${OUT_DIR:-artifacts/benchmarks/warm-start}"
WARM_REQUESTS="${WARM_REQUESTS:-5}"
mkdir -p "$OUT_DIR"

# Warm cache/path before timing.
i=0
while [ "$i" -lt "$WARM_REQUESTS" ]; do
  curl -sS -o /dev/null "$BASE_URL$QUERY"
  i=$((i+1))
done

start_ns=$(date +%s%N)
code=$(curl -sS -o "$OUT_DIR/response.json" -w "%{http_code}" "$BASE_URL$QUERY")
end_ns=$(date +%s%N)

ms=$(( (end_ns - start_ns) / 1000000 ))
printf '{"http_code":%s,"warm_start_ms":%s,"warm_requests":%s}\n' "$code" "$ms" "$WARM_REQUESTS" | tee "$OUT_DIR/result.json"

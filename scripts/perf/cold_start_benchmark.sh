#!/usr/bin/env sh
set -eu

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
QUERY="${QUERY:-/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38}"
OUT_DIR="${OUT_DIR:-artifacts/benchmarks/cold-start}"
mkdir -p "$OUT_DIR"

start_ns=$(date +%s%N)
code=$(curl -sS -o "$OUT_DIR/response.json" -w "%{http_code}" "$BASE_URL$QUERY")
end_ns=$(date +%s%N)

ms=$(( (end_ns - start_ns) / 1000000 ))
printf '{"http_code":%s,"cold_start_ms":%s}\n' "$code" "$ms" | tee "$OUT_DIR/result.json"
